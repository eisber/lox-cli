#define WIN32_LEAN_AND_MEAN
#include <windows.h>

/* Test-only: keep the complete heap-locked walk native, without JVM transitions. */
__declspec(dllexport) BOOL lox_test_heap_live(const void *pointer) {
    return HeapValidate(GetProcessHeap(), 0, pointer);
}

__declspec(dllexport) DWORD lox_test_heap_count(
    const unsigned char *prefix, SIZE_T prefix_length, SIZE_T minimum_bytes,
    SIZE_T *count
) {
    HANDLE heap = GetProcessHeap();
    PROCESS_HEAP_ENTRY entry = {0};
    DWORD error;
    *count = 0;
    if (prefix_length == 0 || minimum_bytes < prefix_length) {
        return ERROR_INVALID_PARAMETER;
    }
    if (!HeapLock(heap)) {
        return GetLastError();
    }
    while (HeapWalk(heap, &entry)) {
        if ((entry.wFlags & PROCESS_HEAP_ENTRY_BUSY) && entry.cbData >= minimum_bytes) {
            const unsigned char *data = (const unsigned char *)entry.lpData;
            SIZE_T i;
            for (i = 0; i < prefix_length && data[i] == prefix[i]; ++i) {}
            if (i == prefix_length) {
                ++*count;
            }
        }
    }
    error = GetLastError();
    if (!HeapUnlock(heap)) {
        return GetLastError();
    }
    return error == ERROR_NO_MORE_ITEMS ? ERROR_SUCCESS : error;
}
