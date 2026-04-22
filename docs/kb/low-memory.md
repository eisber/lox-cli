# Low memory

Source: https://www.loxone.com/enen/kb/low-memory/

---

If the Miniserver does not have enough free memory available, it’s functionality will be limited. [System Status](https://www.loxone.com/enen/kb/systemstatus/) messages provide more information about the cause of the issue.

### **The available memory is low**

The Miniserver no longer has enough free memory for unscheduled tasks (update, request of large data blocks etc.). In this case the basic function is still given, but the Miniserver is already limited in its work.

[Reduce the amount of memory](#reduceMemory) required by the Miniserver to maintain system stability.

### **Auto-Update aborted**

The Auto-Update could not be performed because the Miniserver did not have enough free memory available. For an update, sufficient free memory must be available (at least 10MB), in order for the Miniserver to be able to load the update.

By restarting the Miniserver, the main memory is cleared and the memory requirement is reduced for a short amount of time. After a restart, the Auto-Update is automatically restarted.

[Reduce the memory requirement](#reduceMemory) of the Miniserver so that the stability of the system can also be ensured in the future.

### **Miniserver Reboot**

If too little memory is available for a scheduled tasks of the Miniserver, a safety reboot is performed automatically. This is necessary because the correct function cannot be guaranteed otherwise

If these security restarts occur frequently, [the memory requirement of the Miniserver must be reduced.](#reduceMemory)

### **How to reduce memory requirements**

Memory consumption can usually be optimized by thoughtful configuration. Avoid large custom logic and use [Auto-Configuration](https://www.loxone.com/enen/kb/auto-configuration/). Convert 1st generation function blocks (Lighting Controller, Intelligent Room Controller) to their updated version. The new function blocks are optimized for performance, making a lot of custom logic unnecessary, thus taking up less memory.

The programming can also be spread over several Miniservers ([Gateway/Client with Concentrator](https://www.loxone.com/enen/kb/miniserver-clientgateway-concentrator/)), and by sharing the workload, more memory will be available in total. By adding another Miniserver to the system, existing programming can be transferred from the current Miniserver.