/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2002 Daniel Stenberg
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public License
 * as published by the Free Software Foundation; either version 2
 * of the License, or (at your option) any later version.
 *
 * This software is distributed on an "AS IS" basis, WITHOUT WARRANTY OF ANY
 * KIND, either express or implied.
 *
 ****************************************************************************/
#ifndef _TREE_H_
#define _TREE_H_

#include <stdbool.h>
#include <applimits.h>
#include <file.h>
#include "config.h"
#include "icon.h"

/* keep this struct compatible (total size and name member)
 * with struct tagtree_entry (tagtree.h) */
struct entry {
    char *name;
    int attr; /* FAT attributes + file type flags */
    unsigned time_write; /* Last write time */
    #ifdef HAVE_TAGCACHE
    int customaction; /* db use */
    char* album_name; /* db use */
    #endif
};

#define BROWSE_SELECTONLY       0x0001  /* exit on selecting a file */
#define BROWSE_NO_CONTEXT_MENU  0x0002  /* disable context menu */
#define BROWSE_RUNFILE          0x0004  /* do ft_open() on the file instead of browsing */
#define BROWSE_DIRFILTER        0x0080  /* override global_settings.dirfilter with browse_context.dirfilter */
#define BROWSE_SELECTED         0x0100  /* this bit is set if user selected item */


struct tree_context;

struct tree_cache {
    /* A big buffer with plenty of entry structs, contains all files and dirs
     * in the current dir (with filters applied)
     * Note that they're buflib-allocated and can therefore possibly move
     * They need to be locked if used around yielding functions */
    int     entries_handle;         /* handle to the entry cache */
    int     name_buffer_handle;     /* handle to the name cache */
    int     max_entries;            /* Max entries in the cache */
    int     name_buffer_size;       /* in bytes */
};

struct browse_context {
    int dirfilter;
    unsigned flags;             /* ored BROWSE_* */
    bool (*callback_show_item)(char *name, int attr, struct tree_context *tc);
                                /* callback function to determine to show/hide
                                   the item for custom browser */
    char *title;                /* title of the browser. if set to NULL,
                                   directory name is used. */
    enum themable_icons icon;   /* title icon */
    const char *root;           /* full path of start directory */
    const char *selected;       /* name of selected file in the root */
    char *buf;                  /* buffer to store selected file */
    size_t bufsize;             /* size of the buffer */
    bool disable_gui;          /* disable gui for this browse */
};

/* browser context for file or db */
struct tree_context {
    /* The directory we are browsing */
    char currdir[MAX_PATH];
    /* the number of directories we have crossed from / */
    int dirlevel;
    /* The currently selected file/id3dbitem index (old dircursor+dirfile) */
    int selected_item;
    /* The selected item in each directory crossed
     * (used when we want to return back to a previouws directory)*/
    int selected_item_history[MAX_DIR_LEVELS];

    int *dirfilter; /* file use */
    int filesindir; /* The number of files in the dircache */
    int dirsindir; /* file use */
    int dirlength; /* total number of entries in dir, incl. those not loaded */
#ifdef HAVE_TAGCACHE
    int currtable; /* db use */
    int currextra; /* db use */
    int special_entry_count; /* db use */
#endif
    int sort_dir; /* directory sort order */
    int out_of_tree; /* shortcut from elsewhere */
    struct tree_cache cache;
    bool dirfull;
    bool is_browsing; /* valid browse context? */

    struct browse_context *browse;
};

/*
 * Call one of the two below after yields since the entrys may move inbetween */
struct entry* tree_get_entries(struct tree_context *t);
/* returns NULL on invalid index */
struct entry* tree_get_entry_at(struct tree_context *t, int index);

void tree_mem_init(void) INIT_ATTR;
void tree_init(void) INIT_ATTR;
char* get_current_file(char* buffer, size_t buffer_len);
void set_dirfilter(int l_dirfilter);
void set_current_file(const char *path);
int rockbox_browse(struct browse_context *browse);
int rockbox_browse_root();
int create_playlist(void);
void resume_directory(const char *dir);

void tree_lock_cache(struct tree_context *t);
void tree_unlock_cache(struct tree_context *t);

#ifdef WIN32
/* it takes an int on windows */
#define getcwd_size_t int
#else
#define getcwd_size_t size_t
#endif
char *getcwd(char *buf, getcwd_size_t size);
void reload_directory(void);
bool check_rockboxdir(void);
struct tree_context* tree_get_context(void);
void tree_flush(void);
void tree_restore(void);

bool bookmark_play(char* resume_file, int index, unsigned long elapsed,
                   unsigned long offset, int seed, char *filename);

#endif
