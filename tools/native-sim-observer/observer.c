#include "dr_api.h"
#include "drmgr.h"
#include <string.h>

#define MAX_SITES 16
#define MAX_EVENTS 4096
#define MAX_OBJECTS 512
#define MAX_PREFIX 256
#define CONFIG_BYTES 8192

typedef struct {
    uint id, rva, post, mask;
    app_pc pc;
    bool selected, instrumented;
} site_t;

typedef struct {
    ptr_uint_t address, vtable;
    size_t size, vtable_size;
    uint status, vtable_status;
    byte bytes[MAX_PREFIX], vtable_bytes[8];
} sample_t;

typedef struct {
    uint site, thread;
    ptr_uint_t pc, regs[5], return_address;
    bool context_ok, return_ok;
    sample_t samples[4];
} event_t;

static site_t sites[MAX_SITES];
static event_t events[MAX_EVENTS];
static ptr_uint_t objects[MAX_OBJECTS];
static uint site_count, event_count, object_count;
static uint event_budget, object_budget, prefix;
static uint expected_size, expected_timestamp;
static uint64 dropped, bad_reads, partial_reads, object_drops, context_failures;
static char module_name[128], target_sha[65], client_sha[65], config_sha[65];
static app_pc module_base;
static uint module_size, module_timestamp;
static bool module_seen, module_mismatch, nudge_exit;
static void *lock;
static file_t output = INVALID_FILE;

static void
fail(const char *message)
{
    dr_fprintf(STDERR, "native observer: %s\n", message);
    dr_abort();
}

static uint
read_bytes(ptr_uint_t address, byte *bytes, size_t wanted, size_t *got)
{
    bool ok;
    *got = 0;
    ok = dr_safe_read((const void *)address, wanted, bytes, got);
    if (ok && *got == wanted)
        return 1;
    if (*got != 0) {
        ++partial_reads;
        return 2;
    }
    ++bad_reads;
    return 3;
}

static void
sample_pointer(sample_t *sample, ptr_uint_t address)
{
    uint i;
    sample->address = address;
    for (i = 0; i < object_count; ++i) {
        if (objects[i] == address)
            break;
    }
    if (i == object_count) {
        if (object_count == object_budget) {
            sample->status = 4;
            ++object_drops;
            return;
        }
        objects[object_count++] = address;
    }
    sample->status = read_bytes(address, sample->bytes, prefix, &sample->size);
    if (sample->size >= sizeof(ptr_uint_t)) {
        memcpy(&sample->vtable, sample->bytes, sizeof(ptr_uint_t));
        sample->vtable_status = read_bytes(sample->vtable, sample->vtable_bytes,
                                          sizeof(sample->vtable_bytes),
                                          &sample->vtable_size);
    }
}

static void
observe_entry(site_t *site)
{
    dr_mcontext_t mc = { sizeof(mc), DR_MC_INTEGER | DR_MC_CONTROL };
    void *drcontext = dr_get_current_drcontext();
    event_t *event;
    size_t got;
    uint i;
    dr_mutex_lock(lock);
    if (event_count == event_budget) {
        ++dropped;
        dr_mutex_unlock(lock);
        return;
    }
    event = &events[event_count++];
    event->site = site->id;
    event->thread = (uint)dr_get_thread_id(drcontext);
    event->pc = (ptr_uint_t)site->pc;
    if (!dr_get_mcontext(drcontext, &mc)) {
        ++context_failures;
        dr_mutex_unlock(lock);
        return;
    }
    event->context_ok = true;
    event->regs[0] = mc.xcx;
    event->regs[1] = mc.xdx;
    event->regs[2] = mc.r8;
    event->regs[3] = mc.r9;
    event->regs[4] = mc.xsp;
    event->return_ok = read_bytes(mc.xsp, (byte *)&event->return_address,
                                  sizeof(event->return_address), &got) == 1;
    for (i = 0; i < 4; ++i) {
        if ((site->mask & (1U << i)) != 0)
            sample_pointer(&event->samples[i], event->regs[i]);
    }
    dr_mutex_unlock(lock);
}

static dr_emit_flags_t
insert_entry(void *drcontext, void *tag, instrlist_t *bb, instr_t *instruction,
             bool for_trace, bool translating, void *user_data)
{
    uint i;
    app_pc pc = instr_get_app_pc(instruction);
    (void)tag;
    (void)for_trace;
    (void)translating;
    (void)user_data;
    if (!instr_is_app(instruction) || pc == NULL)
        return DR_EMIT_DEFAULT;
    for (i = 0; i < site_count; ++i) {
        if (sites[i].selected && sites[i].pc == pc) {
            dr_insert_clean_call(drcontext, bb, instruction, (void *)observe_entry,
                                 true, 1, OPND_CREATE_INTPTR(&sites[i]));
            dr_mutex_lock(lock);
            sites[i].instrumented = true;
            dr_mutex_unlock(lock);
            break;
        }
    }
    return DR_EMIT_DEFAULT;
}

static void
module_load(void *drcontext, const module_data_t *mod, bool loaded)
{
    module_data_t *main_module = dr_get_main_module();
    bool is_main = main_module != NULL && main_module->start == mod->start;
    uint i;
    const char *name = mod->names.file_name;
    (void)drcontext;
    (void)loaded;
    if (main_module != NULL)
        dr_free_module_data(main_module);
    if (!is_main)
        return;
    if (module_seen) {
        if (module_base != mod->start || module_size != mod->module_internal_size ||
            module_timestamp != mod->timestamp)
            module_mismatch = true;
        return;
    }
    module_seen = true;
    module_base = mod->start;
    module_size = (uint)mod->module_internal_size;
    module_timestamp = mod->timestamp;
    if (name == NULL || _stricmp(name, module_name) != 0 ||
        module_size != expected_size || module_timestamp != expected_timestamp) {
        module_mismatch = true;
        return;
    }
    for (i = 0; i < site_count; ++i) {
        sites[i].pc = mod->start + sites[i].rva;
        sites[i].selected = true;
    }
}

static void
hex_bytes(const byte *bytes, size_t size)
{
    static const char digits[] = "0123456789abcdef";
    char encoded[MAX_PREFIX * 2 + 1];
    size_t i;
    for (i = 0; i < size; ++i) {
        encoded[i * 2] = digits[bytes[i] >> 4];
        encoded[i * 2 + 1] = digits[bytes[i] & 15];
    }
    encoded[size * 2] = '\0';
    dr_fprintf(output, "%s", encoded);
}

static void
exit_event(void)
{
    uint i, j;
    dr_mutex_lock(lock);
    dr_fprintf(output,
        "{\"type\":\"HEADER\",\"schema\":2,\"kind\":\"passive-entry-observer\","
        "\"backend\":\"drmgr-entry\",\"client_version\":5,\"supports_return\":false,"
        "\"target_sha256\":\"%s\",\"client_sha256\":\"%s\",\"config_sha256\":\"%s\","
        "\"module\":\"%s\",\"module_base\":\"" PFX "\",\"module_size\":%u,"
        "\"module_timestamp\":%u,\"module_seen\":%s,\"module_mismatch\":%s,"
        "\"max_events\":%u,\"max_objects\":%u,\"prefix_bytes\":%u}\n",
        target_sha, client_sha, config_sha, module_name, module_base, module_size,
        module_timestamp, module_seen ? "true" : "false",
        module_mismatch ? "true" : "false", event_budget, object_budget, prefix);
    for (i = 0; i < site_count; ++i) {
        dr_fprintf(output,
            "{\"type\":\"SITE\",\"id\":%u,\"rva\":%u,\"post\":%u,\"mask\":%u,"
            "\"selected\":%s,\"instrumented\":%s}\n", sites[i].id, sites[i].rva,
            sites[i].post, sites[i].mask, sites[i].selected ? "true" : "false",
            sites[i].instrumented ? "true" : "false");
    }
    for (i = 0; i < event_count; ++i) {
        event_t *e = &events[i];
        dr_fprintf(output,
            "{\"type\":\"ENTRY\",\"seq\":%u,\"site\":%u,\"thread\":%u,\"context_ok\":%s",
            i, e->site, e->thread,
            e->context_ok ? "true" : "false");
        if (e->context_ok) {
            dr_fprintf(output,
                ",\"pc\":\"" PFX "\",\"rcx\":\"" PFX "\",\"rdx\":\"" PFX
                "\",\"r8\":\"" PFX "\",\"r9\":\"" PFX "\",\"rsp\":\"" PFX
                "\",\"return_ok\":%s",
                e->pc, e->regs[0], e->regs[1], e->regs[2], e->regs[3], e->regs[4],
                e->return_ok ? "true" : "false");
            if (e->return_ok)
                dr_fprintf(output, ",\"return_address\":\"" PFX "\"", e->return_address);
            dr_fprintf(output, ",\"samples\":[");
            for (j = 0; j < 4; ++j) {
                sample_t *s = &e->samples[j];
                dr_fprintf(output,
                    "%s{\"reg\":%u,\"address\":\"" PFX "\",\"status\":%u,"
                    "\"size\":%u,\"bytes\":\"",
                    j ? "," : "", j, s->address, s->status, (uint)s->size);
                hex_bytes(s->bytes, s->size);
                dr_fprintf(output, "\",\"vtable_status\":%u,\"vtable_size\":%u",
                           s->vtable_status, (uint)s->vtable_size);
                if (s->size >= sizeof(ptr_uint_t)) {
                    dr_fprintf(output, ",\"vtable\":\"" PFX "\"", s->vtable);
                }
                dr_fprintf(output, ",\"vtable_bytes\":\"");
                hex_bytes(s->vtable_bytes, s->vtable_size);
                dr_fprintf(output, "\"}");
            }
            dr_fprintf(output, "]");
        }
        dr_fprintf(output, "}\n");
    }
    dr_fprintf(output,
        "{\"type\":\"SUMMARY\",\"events\":%u,\"unique_objects\":%u,"
        "\"dropped_events\":%llu,\"unreadable_reads\":%llu,\"partial_reads\":%llu,"
        "\"truncated_reads\":%llu,"
        "\"object_limit_reads\":%llu,\"context_failures\":%llu,\"nudge_exit\":%s}\n",
        event_count, object_count, (unsigned long long)dropped,
        (unsigned long long)bad_reads, (unsigned long long)partial_reads,
        (unsigned long long)partial_reads,
        (unsigned long long)object_drops, (unsigned long long)context_failures,
        nudge_exit ? "true" : "false");
    dr_flush_file(output);
    dr_close_file(output);
    dr_mutex_unlock(lock);
    dr_mutex_destroy(lock);
    drmgr_exit();
}

static void
nudge(void *drcontext, uint64 argument)
{
    (void)drcontext;
    if (argument == 1) {
        nudge_exit = true;
        dr_exit_process(0);
    }
}

DR_EXPORT void
dr_client_main(client_id_t id, int argc, const char *argv[])
{
    file_t config;
    char *text, *line, *next;
    ssize_t length;
    uint i, fields;
    dr_module_iterator_t *modules;
    if (argc != 4 || strlen(argv[3]) != 64)
        fail("expected CONFIG OUTPUT CONFIG_SHA256");
    text = dr_global_alloc(CONFIG_BYTES);
    if (text == NULL)
        fail("cannot allocate config buffer");
    config = dr_open_file(argv[1], DR_FILE_READ);
    if (config == INVALID_FILE)
        fail("cannot open config");
    length = dr_read_file(config, text, CONFIG_BYTES - 1);
    dr_close_file(config);
    if (length <= 0 || length == CONFIG_BYTES - 1)
        fail("empty or oversized config");
    text[length] = '\0';
    line = text;
    next = strchr(line, '\n');
    if (next == NULL)
        fail("missing config header");
    *next++ = '\0';
    fields = dr_sscanf(line, "OBS1 %127s %u %u %u %u %u %64s %64s",
                       module_name, &expected_size, &expected_timestamp, &event_budget,
                       &object_budget, &prefix, target_sha, client_sha);
    if (fields != 8 || event_budget == 0 || event_budget > MAX_EVENTS ||
        object_budget == 0 || object_budget > MAX_OBJECTS ||
        prefix < 8 || prefix > MAX_PREFIX || strlen(target_sha) != 64 ||
        strlen(client_sha) != 64)
        fail("invalid config limits/identity");
    while (*next != '\0') {
        site_t *site;
        line = next;
        next = strchr(line, '\n');
        if (next == NULL || site_count == MAX_SITES)
            fail("invalid site count or unterminated line");
        *next++ = '\0';
        site = &sites[site_count];
        if (dr_sscanf(line, "%u %x %u %u", &site->id, &site->rva, &site->post,
                      &site->mask) != 4 || site->rva >= expected_size ||
            site->post != 0 || site->mask > 15)
            fail("invalid site");
        for (i = 0; i < site_count; ++i) {
            if (sites[i].id == site->id || sites[i].rva == site->rva)
                fail("duplicate site");
        }
        ++site_count;
    }
    if (site_count == 0)
        fail("no sites");
    dr_global_free(text, CONFIG_BYTES);
    memcpy(config_sha, argv[3], 65);
    lock = dr_mutex_create();
    if (!drmgr_init())
        fail("extension initialization failed");
    if (!drmgr_register_module_load_event(module_load) ||
        !drmgr_register_exit_event(exit_event) ||
        !drmgr_register_bb_instrumentation_event(NULL, insert_entry, NULL))
        fail("event registration failed");
    dr_register_nudge_event(nudge, id);
    modules = dr_module_iterator_start();
    while (dr_module_iterator_hasnext(modules)) {
        module_data_t *mod = dr_module_iterator_next(modules);
        module_load(dr_get_current_drcontext(), mod, true);
        dr_free_module_data(mod);
    }
    dr_module_iterator_stop(modules);
    output = dr_open_file(argv[2], DR_FILE_WRITE_REQUIRE_NEW);
    if (output == INVALID_FILE)
        fail("output must be new");
}
