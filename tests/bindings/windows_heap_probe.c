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

/* By-value marker: the expected bytes live on the stack, never in the heap
   being measured. A reused address alone is not evidence of retention. */
__declspec(dllexport) DWORD lox_test_heap_status_count(ULONGLONG marker, SIZE_T *count) {
    const ULONGLONG prefix[4] = {
        marker, ~marker, marker ^ 0x5a17c93eb46d208fULL, marker ^ 0x8c26f1a05b73de49ULL
    };
    return lox_test_heap_count((const unsigned char *)prefix, sizeof(prefix), sizeof(prefix), count);
}

/* UniFFI InvalidJson is a big-endian variant tag and two u64 positions.
   Construct the exact small prefix on the stack, not in the measured heap. */
__declspec(dllexport) DWORD lox_test_heap_catalog_error_count(
    ULONGLONG line, ULONGLONG column, SIZE_T *count
) {
    unsigned char prefix[20] = {0, 0, 0, 1};
    unsigned int i;
    for (i = 0; i < 8; ++i) {
        prefix[4 + i] = (unsigned char)(line >> ((7 - i) * 8));
        prefix[12 + i] = (unsigned char)(column >> ((7 - i) * 8));
    }
    return lox_test_heap_count(prefix, sizeof(prefix), sizeof(prefix), count);
}
