#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <intrin.h>
#include <stdbool.h>

typedef struct {
    const uint64_t *vtable;
    uint64_t marker;
    unsigned char fixture_id[16];
} fixture_object;

static const uint64_t table[] = { 0x1122334455667788, 0x8877665544332211 };
static fixture_object object = {
    table, 0x1020304050607080,
    { 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe,
      0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef }
};
static fixture_object frame = {
    table, 0x0908070605040302,
    { 0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87,
      0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e, 0x0f }
};
static volatile uintptr_t last_return_address;
static __declspec(thread) uint64_t tls_marker = 0x1122334455667788;
static __declspec(thread) unsigned int tls_calls;
static __declspec(thread) double tls_value = 1.25;
static volatile LONG fp_calls, fp_ok = 1, tls_ok = 1;
static HANDLE fp_ready, fp_done;
static const char *fp_trigger;
static int matrix_mode;
extern DWORD _tls_index;

typedef struct {
    uintptr_t marker_offset;
    int offset_known;
} tls_snapshot;

static tls_snapshot main_tls, timer_tls;
static char matrix_probe_path[1024];
static int main_probe_reported, timer_probe_reported;

static void
append_text(char *buffer, size_t *position, const char *text)
{
    while (*text != '\0')
        buffer[(*position)++] = *text++;
}

static void
append_hex(char *buffer, size_t *position, uint64_t value)
{
    char digits[16];
    size_t count = 0;
    do {
        digits[count++] = "0123456789abcdef"[value & 15];
        value >>= 4;
    } while (value != 0);
    while (count != 0)
        buffer[(*position)++] = digits[--count];
}

static void
append_field(char *buffer, size_t *position, const char *name, uint64_t value)
{
    append_text(buffer, position, " ");
    append_text(buffer, position, name);
    append_text(buffer, position, "=");
    append_hex(buffer, position, value);
}

static int
read_self(uintptr_t address, void *destination, SIZE_T size)
{
    SIZE_T actual = 0;
    return ReadProcessMemory(GetCurrentProcess(), (LPCVOID)address, destination,
                             size, &actual) && actual == size;
}

static void
report_tls(const char *stage, const char *role, tls_snapshot *snapshot, int initial)
{
    char buffer[512];
    size_t position = 0;
    uintptr_t array = (uintptr_t)__readgsqword(0x58), slot = 0;
    uint64_t marker = 0;
    DWORD written, index = _tls_index;
    int slot_read = read_self(array + (uintptr_t)index * sizeof(uintptr_t),
                              &slot, sizeof(slot));
    int marker_read = 0, dr_query = 0, under_dr = 0;
    HMODULE runtime = GetModuleHandleW(L"dynamorio.dll");
    if (slot_read && slot != 0 && initial) {
        snapshot->marker_offset = (uintptr_t)&tls_marker - slot;
        snapshot->offset_known = 1;
    }
    if (slot_read && slot != 0 && snapshot->offset_known)
        marker_read = read_self(slot + snapshot->marker_offset, &marker, sizeof(marker));
    if (runtime != NULL) {
        union {
            FARPROC address;
            bool (__cdecl *running)(void);
        } query;
        query.address = GetProcAddress(runtime, "dr_app_running_under_dynamorio");
        if (query.address != NULL) {
            dr_query = 1;
            under_dr = query.running() ? 1 : 0;
        }
    }
    append_text(buffer, &position, "TLS1 stage=");
    append_text(buffer, &position, stage);
    append_text(buffer, &position, " role=");
    append_text(buffer, &position, role);
    append_field(buffer, &position, "thread", GetCurrentThreadId());
    append_field(buffer, &position, "teb", __readgsqword(0x30));
    append_field(buffer, &position, "array", array);
    append_field(buffer, &position, "index", index);
    append_field(buffer, &position, "slot_read", (uint64_t)slot_read);
    append_field(buffer, &position, "slot", slot);
    append_field(buffer, &position, "offset_known", (uint64_t)snapshot->offset_known);
    append_field(buffer, &position, "offset", snapshot->marker_offset);
    append_field(buffer, &position, "marker_read", (uint64_t)marker_read);
    append_field(buffer, &position, "marker", marker);
    append_field(buffer, &position, "dr_query", (uint64_t)dr_query);
    append_field(buffer, &position, "under_dr", (uint64_t)under_dr);
    buffer[position++] = '\n';
    if (!WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), buffer, (DWORD)position, &written, NULL) ||
        written != position)
        ExitProcess(11);
}

static void
report_probe(const char *role, tls_snapshot *snapshot, int *reported)
{
    if (matrix_mode && !*reported &&
        GetFileAttributesA(matrix_probe_path) != INVALID_FILE_ATTRIBUTES) {
        report_tls("attached", role, snapshot, 0);
        *reported = 1;
    }
}

int verify_sentinels(void);

__declspec(dllexport) __declspec(noinline) double
observe_fp(double x, double y)
{
    if (tls_marker != 0x1122334455667788ULL + tls_calls)
        tls_ok = 0;
    ++tls_marker;
    ++tls_calls;
    InterlockedIncrement(&fp_calls);
    return x + y + tls_value;
}

static VOID CALLBACK
fp_timer(PVOID context, BOOLEAN fired)
{
    unsigned int i;
    DWORD started = GetTickCount();
    (void)context;
    (void)fired;
    if (matrix_mode)
        report_tls("ready", "timer", &timer_tls, 1);
    SetEvent(fp_ready);
    while (fp_trigger != NULL && GetFileAttributesA(fp_trigger) == INVALID_FILE_ATTRIBUTES) {
        report_probe("timer", &timer_tls, &timer_probe_reported);
        if (GetTickCount() - started > 30000) {
            fp_ok = 0;
            SetEvent(fp_done);
            return;
        }
        Sleep(25);
    }
    if (matrix_mode)
        report_tls("triggered", "timer", &timer_tls, 0);
    for (i = 0; i < 32; ++i) {
        if (!verify_sentinels())
            fp_ok = 0;
    }
    SetEvent(fp_done);
}

__declspec(dllexport) __declspec(noinline) uint64_t
observe_entry(fixture_object *self, uint64_t marker, void *user, fixture_object *context)
{
    last_return_address = (uintptr_t)_ReturnAddress();
    return self->marker ^ marker ^ (uint64_t)(uintptr_t)user ^ context->marker;
}

__declspec(dllexport) __declspec(noinline) uint64_t
observe_clock(void)
{
    return 0x0123456789abcdef;
}

int
main(int argc, char **argv)
{
    unsigned int i;
    uint64_t value = 0;
    uint64_t clock;
    DWORD started = GetTickCount();
    HANDLE queue, timer = NULL;
    matrix_mode = argc == 3 && strcmp(argv[1], "--matrix-trigger") == 0;
    if (matrix_mode) {
        size_t length = strlen(argv[2]);
        if (length + sizeof(".probe") > sizeof(matrix_probe_path))
            return 12;
        memcpy(matrix_probe_path, argv[2], length);
        memcpy(matrix_probe_path + length, ".probe", sizeof(".probe"));
    }
    fp_ready = CreateEventW(NULL, TRUE, FALSE, NULL);
    fp_done = CreateEventW(NULL, TRUE, FALSE, NULL);
    queue = CreateTimerQueue();
    if (argc == 3 && (strcmp(argv[1], "--late-trigger") == 0 || matrix_mode))
        fp_trigger = argv[2];
    if (!fp_ready || !fp_done || !queue ||
        !CreateTimerQueueTimer(&timer, queue, fp_timer, NULL, 0, 0, WT_EXECUTEONLYONCE) ||
        WaitForSingleObject(fp_ready, 5000) != WAIT_OBJECT_0)
        return 9;
    if (argc == 3 && (strcmp(argv[1], "--late-trigger") == 0 || matrix_mode)) {
        if (matrix_mode)
            report_tls("ready", "main", &main_tls, 1);
        printf("fixture-v1 ready\n");
        fflush(stdout);
        while (GetFileAttributesA(argv[2]) == INVALID_FILE_ATTRIBUTES) {
            report_probe("main", &main_tls, &main_probe_reported);
            if (GetTickCount() - started > 20000)
                return 8;
            Sleep(25);
        }
        started = GetTickCount();
        if (matrix_mode)
            report_tls("triggered", "main", &main_tls, 0);
    }
    for (i = 0; i < 80; ++i)
        value = observe_entry(&object, 0x13579bdf2468ace0, (void *)(uintptr_t)1, &frame);
    clock = observe_clock();
    for (i = 0; i < 32; ++i) {
        if (!verify_sentinels())
            fp_ok = 0;
    }
    if (WaitForSingleObject(fp_done, 10000) != WAIT_OBJECT_0)
        return 10;
    printf("fixture-v1 value=%016llx clock=%016llx calls=80 return_rva=%llx fp_sentinels=%ld tls=%ld fp_calls=%ld\n",
           (unsigned long long)value, (unsigned long long)clock,
           (unsigned long long)(last_return_address - (uintptr_t)GetModuleHandleW(NULL)),
           fp_ok, tls_ok, fp_calls);
    fflush(stdout);
    if ((argc == 2 && strcmp(argv[1], "--hold") == 0) ||
        (argc == 3 && strcmp(argv[1], "--late-trigger") == 0)) {
        while (GetTickCount() - started < 30000)
            Sleep(25);
    } else if (!matrix_mode)
        Sleep(1500);
    DeleteTimerQueueEx(queue, INVALID_HANDLE_VALUE);
    CloseHandle(fp_ready);
    CloseHandle(fp_done);
    return value == (object.marker ^ 0x13579bdf2468ace0 ^ 1 ^ frame.marker) &&
            clock == 0x0123456789abcdef && fp_ok && tls_ok && fp_calls == 64 ? 0 : 7;
}
