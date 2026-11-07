#include "dr_api.h"
#ifdef ISOLATION_DRMGR
#include "drmgr.h"
#endif

static void
on_exit(void)
{
#ifdef ISOLATION_DRMGR
    drmgr_exit();
#endif
}

DR_EXPORT void
dr_client_main(client_id_t id, int argc, const char *argv[])
{
    (void)id;
    (void)argc;
    (void)argv;
    dr_set_client_name("Independent TLS isolation control", "");
#ifdef ISOLATION_DRMGR
    if (!drmgr_init())
        dr_abort();
    drmgr_register_exit_event(on_exit);
#else
    dr_register_exit_event(on_exit);
#endif
}
