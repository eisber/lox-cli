#include <windows.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <intrin.h>

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
    SetEvent(fp_ready);
    while (fp_trigger != NULL && GetFileAttributesA(fp_trigger) == INVALID_FILE_ATTRIBUTES) {
        if (GetTickCount() - started > 30000) {
            fp_ok = 0;
            SetEvent(fp_done);
            return;
        }
        Sleep(25);
    }
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
    fp_ready = CreateEventW(NULL, TRUE, FALSE, NULL);
    fp_done = CreateEventW(NULL, TRUE, FALSE, NULL);
    queue = CreateTimerQueue();
    if (argc == 3 && strcmp(argv[1], "--late-trigger") == 0)
        fp_trigger = argv[2];
    if (!fp_ready || !fp_done || !queue ||
        !CreateTimerQueueTimer(&timer, queue, fp_timer, NULL, 0, 0, WT_EXECUTEONLYONCE) ||
        WaitForSingleObject(fp_ready, 5000) != WAIT_OBJECT_0)
        return 9;
    if (argc == 3 && strcmp(argv[1], "--late-trigger") == 0) {
        printf("fixture-v1 ready\n");
        fflush(stdout);
        while (GetFileAttributesA(argv[2]) == INVALID_FILE_ATTRIBUTES) {
            if (GetTickCount() - started > 20000)
                return 8;
            Sleep(25);
        }
        started = GetTickCount();
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
    } else
        Sleep(1500);
    DeleteTimerQueueEx(queue, INVALID_HANDLE_VALUE);
    CloseHandle(fp_ready);
    CloseHandle(fp_done);
    return value == (object.marker ^ 0x13579bdf2468ace0 ^ 1 ^ frame.marker) &&
            clock == 0x0123456789abcdef && fp_ok && tls_ok && fp_calls == 64 ? 0 : 7;
}
