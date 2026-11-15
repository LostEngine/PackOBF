#ifndef PACKOBF_GUI_H
#define PACKOBF_GUI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct PackObfGui PackObfGui;

typedef struct {
    uint8_t compression;         /* 0 Fast, 1 Normal */
    uint8_t shader_compression;  /* 0 None, 1 Minify, 2 Minify and obfuscate */
    uint8_t sound_quality;       /* 0 Dynamic, 1 through 12 for -1 through 10 */
    uint8_t target_version;      /* 0 Any, otherwise the Minecraft pack format */
    uint8_t rename_files;
    uint8_t block_unzipping;
    uint8_t corrupt_png_files;
    uint32_t worker_threads;     /* 0 Automatic, or 1 through 256 */
} PackObfGuiSettings;

typedef struct {
    char *status;
    char *summary;               /* Non-NULL once per successful job */
    size_t log_count;
    uint8_t running;
    uint8_t can_save;
} PackObfGuiSnapshot;

typedef struct {
    char *line;                  /* NULL when the index is out of range */
    uint8_t level;               /* 0 Info, 1 Warning, 2 Error */
} PackObfGuiLogEntry;

/* Strings are NUL-terminated UTF-8. Returned strings are owned by the caller
 * and must be freed individually with packobf_gui_free_string. Boolean fields
 * use zero for false and nonzero for true.
 *
 * Serialize calls on each handle, and pass a live handle to all instance
 * functions. Destroy it once when finished. The worker may still be completing
 * a job at destruction. Keep the native library loaded until process exit.
 */
PackObfGuiSettings packobf_gui_defaults(void);
PackObfGui *packobf_gui_create(void);
void packobf_gui_destroy(PackObfGui *app);

/* Start and save return NULL on success or an owned error message. Cache may
 * be NULL. A successful start clears previously collected logs and output.
 * Poll on the frontend event loop to collect pending progress and logs.
 */
char *packobf_gui_start(PackObfGui *app, const char *input, const char *cache,
                       PackObfGuiSettings settings);
void packobf_gui_invalidate(PackObfGui *app);
PackObfGuiSnapshot packobf_gui_poll(PackObfGui *app);
PackObfGuiLogEntry packobf_gui_log(const PackObfGui *app, size_t index);
char *packobf_gui_copyable_logs(const PackObfGui *app);
char *packobf_gui_save(const PackObfGui *app, const char *output);
void packobf_gui_free_string(char *string);

#ifdef __cplusplus
}
#endif

#endif
