/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * C compat layer for rb_* symbols normally provided by zig/src/main.zig.
 *
 * On the headless host the firmware is compiled as static .a files and
 * linked by Zig, so the Zig wrappers ARE present. However, this file is
 * also compiled into libfirmware.a as a fallback for the build to produce
 * the right symbols when Zig's wrappers are not yet available.
 *
 * In practice Zig provides these at link time and the C definitions here
 * are overridden (duplicates are silently dropped by the linker with the
 * --allow-multiple-definition / -multiply_defined suppress flags used by
 * the headless zig build). Keep in sync with zig/src/rockbox/*.zig and
 * firmware/target/hosted/android/cdylib/rb_zig_compat.c.
 */

#include <stdbool.h>
#include <stddef.h>
#include <string.h>

#include "config.h"
#include "playlist.h"
#include "tree.h"
#include "metadata.h"

/* ── Playlist wrappers ──────────────────────────────────────────────────── */

struct playlist_track_info rb_get_track_info_from_current_playlist(int index)
{
    struct playlist_info *pl = playlist_get_current();
    struct playlist_track_info info;
    memset(&info, 0, sizeof(info));
    playlist_get_track_info(pl, index, &info);
    return info;
}

int rb_build_playlist(const char *const *files, int start_index, int size)
{
    struct playlist_info *pl = playlist_get_current();
    struct playlist_insert_context ctx;
    if (playlist_insert_context_create(pl, &ctx,
                                       PLAYLIST_REPLACE, false, false) < 0)
        return -1;
    for (int i = 0; i < size; i++) {
        if (playlist_insert_context_add(&ctx, files[i]) < 0)
            break;
    }
    playlist_insert_context_release(&ctx);
    return start_index;
}

int rb_playlist_insert_tracks(const char *const *files, int position, int size)
{
    struct playlist_info *pl = playlist_get_current();
    struct playlist_insert_context ctx;
    if (playlist_insert_context_create(pl, &ctx, position, false, false) < 0)
        return -1;
    for (int i = 0; i < size; i++) {
        if (playlist_insert_context_add(&ctx, files[i]) < 0)
            break;
    }
    playlist_insert_context_release(&ctx);
    return position;
}

int rb_playlist_insert_track(const char *filename, int position,
                              bool queue, bool sync)
{
    return playlist_insert_track(playlist_get_current(),
                                 filename, position, queue, sync);
}

int rb_playlist_delete_track(int index)
{
    return playlist_delete(playlist_get_current(), index);
}

int rb_playlist_insert_directory(const char *dir, int position,
                                 bool queue, bool recurse)
{
    return playlist_insert_directory(playlist_get_current(),
                                     dir, position, queue, recurse, NULL);
}

int rb_playlist_remove_all_tracks(void)
{
    return playlist_remove_all_tracks(playlist_get_current());
}

int rb_playlist_index(void)               { return playlist_get_current()->index; }
int rb_playlist_first_index(void)         { return playlist_get_current()->first_index; }
int rb_playlist_last_insert_pos(void)     { return playlist_get_current()->last_insert_pos; }
int rb_playlist_seed(void)                { return playlist_get_current()->seed; }
int rb_playlist_last_shuffled_start(void) { return playlist_get_current()->last_shuffled_start; }
int rb_max_playlist_size(void)            { return playlist_get_current()->max_playlist_size; }

/* ── Tree wrappers ──────────────────────────────────────────────────────── */

int rb_rockbox_browse(void)
{
    struct browse_context bc;
    memset(&bc, 0, sizeof(bc));
    return rockbox_browse(&bc);
}

struct tree_context rb_tree_get_context(void)
{
    return *tree_get_context();
}

struct entry *rb_tree_get_entries(void)
{
    return tree_get_entries(tree_get_context());
}

struct entry rb_tree_get_entry_at(int index)
{
    return *tree_get_entry_at(tree_get_context(), index);
}

/* ── Metadata wrapper ───────────────────────────────────────────────────── */

struct mp3entry rb_get_metadata(int fd, const char *trackname)
{
    struct mp3entry id3;
    memset(&id3, 0, sizeof(id3));
    get_metadata(&id3, fd, trackname);
    return id3;
}

/* ── Settings wrapper ───────────────────────────────────────────────────── */

#include "settings.h"
extern struct user_settings global_settings;

int rb_get_crossfade_mode(void)
{
    return global_settings.crossfade;
}
