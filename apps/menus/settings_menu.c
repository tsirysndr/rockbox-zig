/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2007 Jonathan Gordon
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

#include <stdbool.h>
#include <stddef.h>
#include <limits.h>
#include <string.h>
#include "config.h"
#include "lang.h"
#include "action.h"
#include "settings.h"
#include "rbpaths.h"
#include "menu.h"
#include "open_plugin.h"
#include "keyboard.h"
#include "sound_menu.h"
#include "exported_menus.h"
#include "tree.h"
#include "tagtree.h"
#include "usb.h"
#include "splash.h"
#include "yesno.h"
#include "talk.h"
#include "powermgmt.h"
#include "playback.h"
#if CONFIG_RTC
#include "screens.h"
#endif
#include "quickscreen.h"
#ifdef HAVE_DIRCACHE
#include "dircache.h"
#endif
#ifndef HAS_BUTTON_HOLD
#include "mask_select.h"
#endif
#if defined(DX50) || defined(DX90)
#include "governor-ibasso.h"
#include "usb-ibasso.h"
#endif
#include "plugin.h"
#include "onplay.h"
#include "misc.h"

#ifndef HAS_BUTTON_HOLD
static int selectivesoftlock_callback(int action,
                                      const struct menu_item_ex *this_item,
                                      struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;

    switch (action)
    {
        case ACTION_STD_MENU:
        case ACTION_STD_CANCEL:
        case ACTION_EXIT_MENUITEM:
            set_selective_softlock_actions(
                            global_settings.bt_selective_softlock_actions,
                            global_settings.bt_selective_softlock_actions_mask);
            action_autosoftlock_init();
            break;
    }

    return action;
}

static int selectivesoftlock_set_mask(void* param)
{
    (void)param;
int mask = global_settings.bt_selective_softlock_actions_mask;
            struct s_mask_items maskitems[]={
                                       {ID2P(LANG_ACTION_VOLUME), SEL_ACTION_VOL},
                                       {ID2P(LANG_ACTION_PLAY),   SEL_ACTION_PLAY},
                                       {ID2P(LANG_ACTION_SEEK),   SEL_ACTION_SEEK},
                                       {ID2P(LANG_ACTION_SKIP),   SEL_ACTION_SKIP},
 #ifdef HAVE_BACKLIGHT
                                       {ID2P(LANG_ACTION_AUTOLOCK_ON),    SEL_ACTION_AUTOLOCK},
                                       {ID2P(LANG_ACTION_ALWAYSAUTOLOCK), SEL_ACTION_ALWAYSAUTOLOCK},
 #endif
 #if defined(HAVE_TOUCHPAD) || defined(HAVE_TOUCHSCREEN)
                                       {ID2P(LANG_ACTION_DISABLE_TOUCH),  SEL_ACTION_NOTOUCH},
 #endif
                                       {ID2P(LANG_ACTION_DISABLE_NOTIFY), SEL_ACTION_NONOTIFY},
                                       {ID2P(LANG_SOFTLOCK_DISABLE_ALL_NOTIFY), SEL_ACTION_ALLNONOTIFY}
                                            };

            mask = mask_select(mask, ID2P(LANG_SOFTLOCK_SELECTIVE)
                               , maskitems,ARRAYLEN(maskitems));

            if (mask == SEL_ACTION_NONE)
                global_settings.bt_selective_softlock_actions = false;
            else if (global_settings.bt_selective_softlock_actions_mask != mask)
                global_settings.bt_selective_softlock_actions = true;

            global_settings.bt_selective_softlock_actions_mask = mask;

    return true;
}

#endif /* !HAS_BUTTON_HOLD */

/***********************************/
/*    TAGCACHE MENU                */
#ifdef HAVE_TAGCACHE

static void tagcache_rebuild_with_splash(void)
{
    tagcache_rebuild();
    splash(HZ*2, ID2P(LANG_TAGCACHE_FORCE_UPDATE_SPLASH));
}

static void tagcache_update_with_splash(void)
{
    tagcache_update();
    splash(HZ*2, ID2P(LANG_TAGCACHE_FORCE_UPDATE_SPLASH));
}

static int dirs_to_scan(void)
{
    if(plugin_load(VIEWERS_DIR"/db_folder_select.rock", NULL) > PLUGIN_OK)
    {
        static const char *lines[] = {ID2P(LANG_TAGCACHE_BUSY),
                                      ID2P(LANG_TAGCACHE_FORCE_UPDATE)};
        static const struct text_message message = {lines, 2};

        if (gui_syncyesno_run(&message, NULL, NULL) == YESNO_YES)
            tagcache_rebuild_with_splash();
    }
    return 0;
}

#ifdef HAVE_TC_RAMCACHE
MENUITEM_SETTING(tagcache_ram, &global_settings.tagcache_ram, NULL);
#endif
MENUITEM_SETTING(tagcache_autoupdate, &global_settings.tagcache_autoupdate, NULL);
MENUITEM_FUNCTION(tc_init, 0, ID2P(LANG_TAGCACHE_FORCE_UPDATE),
                  (int(*)(void))tagcache_rebuild_with_splash, NULL, Icon_NOICON);
MENUITEM_FUNCTION(tc_update, 0, ID2P(LANG_TAGCACHE_UPDATE),
                  (int(*)(void))tagcache_update_with_splash, NULL, Icon_NOICON);
MENUITEM_SETTING(runtimedb, &global_settings.runtimedb, NULL);

MENUITEM_FUNCTION(tc_export, 0, ID2P(LANG_TAGCACHE_EXPORT),
                  tagtree_export,
                  NULL, Icon_NOICON);

MENUITEM_FUNCTION(tc_import, 0, ID2P(LANG_TAGCACHE_IMPORT),
                  tagtree_import,
                  NULL, Icon_NOICON);
MENUITEM_FUNCTION(tc_paths, 0, ID2P(LANG_SELECT_DATABASE_DIRS),
                  dirs_to_scan, NULL, Icon_NOICON);

MAKE_MENU(tagcache_menu, ID2P(LANG_TAGCACHE), 0, Icon_NOICON,
#ifdef HAVE_TC_RAMCACHE
                &tagcache_ram,
#endif
                &tagcache_autoupdate, &tc_init, &tc_update, &runtimedb,
                &tc_export, &tc_import, &tc_paths
                );
#endif /* HAVE_TAGCACHE */
/*    TAGCACHE MENU                */
/***********************************/

/***********************************/
/*    FILE VIEW MENU               */
MENUITEM_SETTING(sort_case, &global_settings.sort_case, NULL);
MENUITEM_SETTING(sort_dir, &global_settings.sort_dir, NULL);
MENUITEM_SETTING(sort_file, &global_settings.sort_file, NULL);
MENUITEM_SETTING(interpret_numbers, &global_settings.interpret_numbers, NULL);
MENUITEM_SETTING(dirfilter, &global_settings.dirfilter, NULL);
MENUITEM_SETTING(show_filename_ext, &global_settings.show_filename_ext, NULL);
MENUITEM_SETTING(browse_current, &global_settings.browse_current, NULL);
MENUITEM_SETTING(show_path_in_browser, &global_settings.show_path_in_browser, NULL);
#ifdef HAVE_HOTKEY
MENUITEM_SETTING(hotkey_tree_item, &global_settings.hotkey_tree, NULL);
#endif
static int clear_start_directory(void)
{
    path_append(global_settings.start_directory, PATH_ROOTSTR,
                PA_SEP_HARD, sizeof(global_settings.start_directory));
    settings_save();
    splash(HZ, ID2P(LANG_RESET_DONE_CLEAR));
    return false;
}
MENUITEM_FUNCTION(clear_start_directory_item, 0, ID2P(LANG_RESET_START_DIR),
                  clear_start_directory, NULL, Icon_file_view_menu);

static int filemenu_callback(int action,
                             const struct menu_item_ex *this_item,
                             struct gui_synclist *this_list);
MAKE_MENU(file_menu, ID2P(LANG_FILE), filemenu_callback, Icon_file_view_menu,
                &sort_case, &sort_dir, &sort_file, &interpret_numbers,
                &dirfilter, &show_filename_ext, &browse_current,
                &show_path_in_browser,
                &clear_start_directory_item
#ifdef HAVE_HOTKEY
                ,&hotkey_tree_item
#endif
                );
static int filemenu_callback(int action,
                             const struct menu_item_ex *this_item,
                             struct gui_synclist *this_list)
{
    (void)this_list;

    /* Show File View menu in Settings or File Browser,
       but not in Database or Playlist Catalog */
    if (action == ACTION_REQUEST_MENUITEM &&
        this_item == &file_menu &&
        get_current_activity() != ACTIVITY_SETTINGS &&
        (get_onplay_context() != CONTEXT_TREE
         || *tree_get_context()->dirfilter == SHOW_M3U))
        return ACTION_EXIT_MENUITEM;

    return action;
}

/*    FILE VIEW MENU               */
/***********************************/


/***********************************/
/*    SYSTEM MENU                  */

/* Battery */
#if BATTERY_CAPACITY_INC > 0
MENUITEM_SETTING(battery_capacity, &global_settings.battery_capacity, NULL);
#endif

#ifdef HAVE_USB_CHARGING_ENABLE
static int usbcharging_callback(int action,
                                const struct menu_item_ex *this_item,
                                struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;
    switch (action)
    {
        case ACTION_EXIT_MENUITEM: /* on exit */
            usb_charging_enable(global_settings.usb_charging);
            break;
    }
    return action;
}
MENUITEM_SETTING(usb_charging, &global_settings.usb_charging, usbcharging_callback);
#endif /* HAVE_USB_CHARGING_ENABLE */
MAKE_MENU(battery_menu, ID2P(LANG_BATTERY_MENU), 0, Icon_NOICON,
#if BATTERY_CAPACITY_INC > 0
            &battery_capacity,
#endif
#ifdef HAVE_USB_CHARGING_ENABLE
            &usb_charging,
#endif
         );
#if defined(DX50) || defined(DX90) || (defined(HAVE_USB_POWER) && !defined(USB_NONE) && !defined(SIMULATOR))
MENUITEM_SETTING(usb_mode, &global_settings.usb_mode, NULL);
#endif
/* Disk */
#ifdef HAVE_DISK_STORAGE
MENUITEM_SETTING(disk_spindown, &global_settings.disk_spindown, NULL);
#endif
#ifdef HAVE_DIRCACHE
static int dircache_callback(int action,
                             const struct menu_item_ex *this_item,
                             struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;
    switch (action)
    {
        case ACTION_EXIT_MENUITEM: /* on exit */
            if (global_settings.dircache)
            {
                if (dircache_enable() < 0)
                    splash(HZ*2, ID2P(LANG_PLEASE_REBOOT));
            }
            else
            {
                dircache_disable();
            }
            break;
    }
    return action;
}
MENUITEM_SETTING(dircache, &global_settings.dircache, dircache_callback);
#endif
#if defined(HAVE_DIRCACHE) || defined(HAVE_DISK_STORAGE)
MAKE_MENU(disk_menu, ID2P(LANG_DISK_MENU), 0, Icon_NOICON,
#ifdef HAVE_DISK_STORAGE
          &disk_spindown,
#endif
#ifdef HAVE_DIRCACHE
            &dircache,
#endif
         );
#endif

/* Limits menu */
MENUITEM_SETTING(max_files_in_dir, &global_settings.max_files_in_dir, NULL);
MENUITEM_SETTING(max_files_in_playlist, &global_settings.max_files_in_playlist, NULL);
MENUITEM_SETTING(default_glyphs, &global_settings.glyphs_to_cache, NULL);
MAKE_MENU(limits_menu, ID2P(LANG_LIMITS_MENU), 0, Icon_NOICON,
           &max_files_in_dir, &max_files_in_playlist
           ,&default_glyphs
           );

#ifdef HAVE_PERCEPTUAL_VOLUME
/* Volume adjustment */
MENUITEM_SETTING(volume_adjust_mode, &global_settings.volume_adjust_mode, NULL);
MENUITEM_SETTING(volume_adjust_norm_steps, &global_settings.volume_adjust_norm_steps, NULL);
#endif

/* Keyclick menu */
MENUITEM_SETTING(keyclick, &global_settings.keyclick, NULL);
MENUITEM_SETTING(keyclick_repeats, &global_settings.keyclick_repeats, NULL);
#ifdef HAVE_HARDWARE_CLICK
MENUITEM_SETTING(keyclick_hardware, &global_settings.keyclick_hardware, NULL);
MAKE_MENU(keyclick_menu, ID2P(LANG_KEYCLICK), 0, Icon_NOICON,
           &keyclick, &keyclick_hardware, &keyclick_repeats);
#else
MAKE_MENU(keyclick_menu, ID2P(LANG_KEYCLICK), 0, Icon_NOICON,
           &keyclick, &keyclick_repeats);
#endif

#if CONFIG_CHARGING
MENUITEM_SETTING(car_adapter_mode, &global_settings.car_adapter_mode, NULL);
MENUITEM_SETTING(car_adapter_mode_delay, &global_settings.car_adapter_mode_delay, NULL);
MAKE_MENU(car_adapter_mode_menu, ID2P(LANG_CAR_ADAPTER_MODE), 0, Icon_NOICON,
           &car_adapter_mode, &car_adapter_mode_delay);
#endif
#ifdef IPOD_ACCESSORY_PROTOCOL
MENUITEM_SETTING(serial_bitrate, &global_settings.serial_bitrate, NULL);
#endif
#ifdef HAVE_ACCESSORY_SUPPLY
MENUITEM_SETTING(accessory_supply, &global_settings.accessory_supply, NULL);
#endif
#ifdef HAVE_LINEOUT_POWEROFF
MENUITEM_SETTING(lineout_onoff, &global_settings.lineout_active, NULL);
#endif
#ifdef USB_ENABLE_HID
MENUITEM_SETTING(usb_hid, &global_settings.usb_hid, NULL);
MENUITEM_SETTING(usb_keypad_mode, &global_settings.usb_keypad_mode, NULL);
#endif
#if defined(USB_ENABLE_STORAGE) && defined(HAVE_MULTIDRIVE)
MENUITEM_SETTING(usb_skip_first_drive, &global_settings.usb_skip_first_drive, NULL);
#endif

#ifdef HAVE_MORSE_INPUT
MENUITEM_SETTING(morse_input, &global_settings.morse_input, NULL);
#endif

#ifdef HAVE_BUTTON_LIGHT
MENUITEM_SETTING(buttonlight_timeout, &global_settings.buttonlight_timeout, NULL);
#endif

#ifdef HAVE_BUTTONLIGHT_BRIGHTNESS
MENUITEM_SETTING(buttonlight_brightness, &global_settings.buttonlight_brightness, NULL);
#endif

#ifdef HAVE_TOUCHPAD_SENSITIVITY_SETTING
MENUITEM_SETTING(touchpad_sensitivity, &global_settings.touchpad_sensitivity, NULL);
#endif

#ifdef HAVE_TOUCHPAD_DEADZONE
MENUITEM_SETTING(touchpad_deadzone, &global_settings.touchpad_deadzone, NULL);
#endif

#ifdef HAVE_QUICKSCREEN
MENUITEM_SETTING(shortcuts_replaces_quickscreen, &global_settings.shortcuts_replaces_qs, NULL);
#endif

#ifndef HAS_BUTTON_HOLD

MENUITEM_SETTING(bt_selective_actions,
                 &global_settings.bt_selective_softlock_actions,
                                                    selectivesoftlock_callback);
MENUITEM_FUNCTION(sel_softlock_mask, 0, ID2P(LANG_SETTINGS),
	              selectivesoftlock_set_mask, selectivesoftlock_callback,
	              Icon_Menu_setting);

MAKE_MENU(sel_softlock, ID2P(LANG_SOFTLOCK_SELECTIVE),
          NULL, Icon_Menu_setting, &bt_selective_actions, &sel_softlock_mask);
#endif /* !HAS_BUTTON_HOLD */

#if defined(DX50) || defined(DX90)
MENUITEM_SETTING(governor, &global_settings.governor, NULL);
#endif

MAKE_MENU(system_menu, ID2P(LANG_SYSTEM),
          0, Icon_System_menu,
#if (BATTERY_CAPACITY_INC > 0) || defined(HAVE_USB_CHARGING_ENABLE)
            &battery_menu,
#endif
#if defined(HAVE_DIRCACHE) || defined(HAVE_DISK_STORAGE)
            &disk_menu,
#endif
            &limits_menu,
#ifdef HAVE_PERCEPTUAL_VOLUME
            &volume_adjust_mode,
            &volume_adjust_norm_steps,
#endif
#ifdef HAVE_QUICKSCREEN
            &shortcuts_replaces_quickscreen,
#endif
#ifdef HAVE_MORSE_INPUT
            &morse_input,
#endif
#if CONFIG_CHARGING
            &car_adapter_mode_menu,
#endif
#ifdef IPOD_ACCESSORY_PROTOCOL
            &serial_bitrate,
#endif
#ifdef HAVE_ACCESSORY_SUPPLY
            &accessory_supply,
#endif
#ifdef HAVE_LINEOUT_POWEROFF
            &lineout_onoff,
#endif
#ifdef HAVE_BUTTON_LIGHT
            &buttonlight_timeout,
#endif
#ifdef HAVE_BUTTONLIGHT_BRIGHTNESS
            &buttonlight_brightness,
#endif
            &keyclick_menu,
#ifdef HAVE_TOUCHPAD_SENSITIVITY_SETTING
            &touchpad_sensitivity,
#endif
#ifdef HAVE_TOUCHPAD_DEADZONE
            &touchpad_deadzone,
#endif
#ifndef HAS_BUTTON_HOLD
            &sel_softlock,
#endif
#ifdef USB_ENABLE_HID
            &usb_hid,
            &usb_keypad_mode,
#endif
#if defined(USB_ENABLE_STORAGE) && defined(HAVE_MULTIDRIVE)
            &usb_skip_first_drive,
#endif

#if defined(DX50) || defined(DX90)
            &governor,
#endif
#if defined(DX50) || defined(DX90) || (defined(HAVE_USB_POWER) && !defined(USB_NONE) && !defined(SIMULATOR))
            &usb_mode,
#endif
         );

/*    SYSTEM MENU                  */
/***********************************/

/***********************************/
/*    STARTUP/SHUTDOWN MENU      */


char* sleeptimer_getname(int selected_item, void * data,
                         char *buffer, size_t buffer_len)
{
    (void)selected_item;
    (void)data;
    return string_sleeptimer(buffer, buffer_len);
}

int sleeptimer_voice(int selected_item, void*data)
{
    (void)selected_item;
    (void)data;
    talk_sleeptimer(-1);
    return 0;
}

/* Handle restarting a current sleep timer to the newly set default
   duration */
static int sleeptimer_duration_cb(int action,
                                  const struct menu_item_ex *this_item,
                                  struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;
    static int initial_duration;
    switch (action)
    {
        case ACTION_ENTER_MENUITEM:
            initial_duration = global_settings.sleeptimer_duration;
            break;
        case ACTION_EXIT_MENUITEM:
            if (initial_duration != global_settings.sleeptimer_duration
                    && get_sleep_timer())
                set_sleeptimer_duration(global_settings.sleeptimer_duration);
    }
    return action;
}

MENUITEM_SETTING(start_screen, &global_settings.start_in_screen, NULL);
MENUITEM_SETTING(poweroff, &global_settings.poweroff, NULL);
MENUITEM_FUNCTION_DYNTEXT(sleeptimer_toggle, 0, toggle_sleeptimer,
                          sleeptimer_getname, sleeptimer_voice, NULL,
                          NULL, Icon_NOICON);
MENUITEM_SETTING(sleeptimer_duration,
                 &global_settings.sleeptimer_duration,
                 sleeptimer_duration_cb);
MENUITEM_SETTING(sleeptimer_on_startup,
                 &global_settings.sleeptimer_on_startup, NULL);
MENUITEM_SETTING(keypress_restarts_sleeptimer,
                 &global_settings.keypress_restarts_sleeptimer, NULL);
MENUITEM_SETTING(show_shutdown_message, &global_settings.show_shutdown_message, NULL);

#if defined(BUTTON_REC) || \
    (CONFIG_KEYPAD == GIGABEAT_PAD) || \
    (CONFIG_KEYPAD == IPOD_4G_PAD) || \
    (CONFIG_KEYPAD == IRIVER_H10_PAD)
#define SETTINGS_CLEAR_ON_HOLD
MENUITEM_SETTING(clear_settings_on_hold,
                 &global_settings.clear_settings_on_hold, NULL);
#endif

MAKE_MENU(startup_shutdown_menu, ID2P(LANG_STARTUP_SHUTDOWN),
          0, Icon_System_menu,
            &show_shutdown_message,
            &start_screen,
            &poweroff,
            &sleeptimer_toggle,
            &sleeptimer_duration,
            &sleeptimer_on_startup,
            &keypress_restarts_sleeptimer,
#if defined(SETTINGS_CLEAR_ON_HOLD)
            &clear_settings_on_hold,
#undef SETTINGS_CLEAR_ON_HOLD
#endif
         );

/*    STARTUP/SHUTDOWN MENU      */
/***********************************/

/***********************************/
/*    BOOKMARK MENU                */
static int bmark_callback(int action,
                          const struct menu_item_ex *this_item,
                          struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;
    switch (action)
    {
        case ACTION_EXIT_MENUITEM: /* on exit */
            if(global_settings.autocreatebookmark ==  BOOKMARK_RECENT_ONLY_YES ||
               global_settings.autocreatebookmark ==  BOOKMARK_RECENT_ONLY_ASK)
            {
                if(global_settings.usemrb == BOOKMARK_NO)
                    global_settings.usemrb = BOOKMARK_YES;

            }
            break;
    }
    return action;
}
MENUITEM_SETTING(autocreatebookmark,
                 &global_settings.autocreatebookmark, bmark_callback);
MENUITEM_SETTING(autoupdatebookmark, &global_settings.autoupdatebookmark, NULL);
MENUITEM_SETTING(autoloadbookmark, &global_settings.autoloadbookmark, NULL);
MENUITEM_SETTING(usemrb, &global_settings.usemrb, NULL);
MAKE_MENU(bookmark_settings_menu, ID2P(LANG_BOOKMARK_SETTINGS), 0,
          Icon_Bookmark,
          &autocreatebookmark, &autoupdatebookmark, &autoloadbookmark, &usemrb);
/*    BOOKMARK MENU                */
/***********************************/

/***********************************/
/*    AUTORESUME MENU              */
#ifdef HAVE_TAGCACHE

static int autoresume_callback(int action,
                               const struct menu_item_ex *this_item,
                               struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;

    if (action == ACTION_EXIT_MENUITEM  /* on exit */
        && global_settings.autoresume_enable
        && !tagcache_is_usable())
    {
        static const char *lines[] = {ID2P(LANG_TAGCACHE_BUSY),
                                      ID2P(LANG_TAGCACHE_FORCE_UPDATE)};
        static const struct text_message message = {lines, 2};

        if (gui_syncyesno_run(&message, NULL, NULL) == YESNO_YES)
            tagcache_rebuild_with_splash();
    }
    return action;
}

static int autoresume_nexttrack_callback(int action,
                                         const struct menu_item_ex *this_item,
                                         struct gui_synclist *this_list)
{
    (void)this_item;
    (void)this_list;
    static int oldval = 0;
    switch (action)
    {
        case ACTION_ENTER_MENUITEM:
            oldval = global_settings.autoresume_automatic;
            break;
        case ACTION_EXIT_MENUITEM:
            if (global_settings.autoresume_automatic == AUTORESUME_NEXTTRACK_CUSTOM
                && plugin_load(VIEWERS_DIR"/db_folder_select.rock",
                               str(LANG_AUTORESUME)) == PLUGIN_OK)
            {
                global_settings.autoresume_automatic = oldval;
            }
    }
    return action;
}

MENUITEM_SETTING(autoresume_enable, &global_settings.autoresume_enable,
                 autoresume_callback);
MENUITEM_SETTING(autoresume_automatic, &global_settings.autoresume_automatic,
                 autoresume_nexttrack_callback);

MAKE_MENU(autoresume_menu, ID2P(LANG_AUTORESUME),
          0, Icon_NOICON,
          &autoresume_enable, &autoresume_automatic);

#endif /* HAVE_TAGCACHE */
/*    AUTORESUME MENU              */
/***********************************/

/***********************************/
/*    VOICE MENU                   */
static int talk_callback(int action,
                         const struct menu_item_ex *this_item,
                         struct gui_synclist *this_list);

MENUITEM_SETTING(talk_menu_item, &global_settings.talk_menu, NULL);
MENUITEM_SETTING(talk_dir_item, &global_settings.talk_dir, NULL);
MENUITEM_SETTING(talk_dir_clip_item, &global_settings.talk_dir_clip, talk_callback);
MENUITEM_SETTING(talk_file_item, &global_settings.talk_file, NULL);
MENUITEM_SETTING(talk_file_clip_item, &global_settings.talk_file_clip, talk_callback);
static int talk_callback(int action,
                         const struct menu_item_ex *this_item,
                         struct gui_synclist *this_list)
{
    (void)this_list;
    static int oldval = 0;
    switch (action)
    {
        case ACTION_ENTER_MENUITEM:
            oldval = global_settings.talk_file_clip;
            break;
        case ACTION_EXIT_MENUITEM:
#ifdef HAVE_CROSSFADE
            audio_set_crossfade(global_settings.crossfade);
#endif
            if (this_item == &talk_dir_clip_item)
                break;
            if (!oldval && global_settings.talk_file_clip)
            {
                /* force reload if newly talking thumbnails,
                because the clip presence is cached only if enabled */
                reload_directory();
            }
            break;
    }
    return action;
}
MENUITEM_SETTING(talk_filetype_item, &global_settings.talk_filetype, NULL);
MENUITEM_SETTING(talk_battery_level_item,
                 &global_settings.talk_battery_level, NULL);
MENUITEM_SETTING(talk_mixer_amp_item, &global_settings.talk_mixer_amp, NULL);
MAKE_MENU(voice_settings_menu, ID2P(LANG_VOICE), 0, Icon_Voice,
          &talk_menu_item, &talk_dir_item, &talk_dir_clip_item,
          &talk_file_item, &talk_file_clip_item, &talk_filetype_item,
          &talk_battery_level_item, &talk_mixer_amp_item);
/*    VOICE MENU                   */
/***********************************/

/*    WPS_CONTEXT_PLUGIN           */
/***********************************/
static void wps_plugin_cb(void)
{
    open_plugin_browse(ID2P(LANG_OPEN_PLUGIN_SET_WPS_CONTEXT_PLUGIN));
}
MENUITEM_FUNCTION(wps_set_context_plugin, 0,
                  ID2P(LANG_OPEN_PLUGIN_SET_WPS_CONTEXT_PLUGIN),
                  wps_plugin_cb, NULL, Icon_Plugin);

/*    WPS_CONTEXT_PLUGIN           */
/***********************************/

/***********************************/
/*    WPS Settings MENU            */

MENUITEM_SETTING(browser_default,
                 &global_settings.browser_default, NULL);

#ifdef HAVE_HOTKEY
MENUITEM_SETTING(hotkey_wps_item, &global_settings.hotkey_wps, NULL);
#endif

MAKE_MENU(wps_settings, ID2P(LANG_WPS), 0, Icon_Playback_menu
            ,&browser_default
#ifdef HAVE_HOTKEY
            ,&hotkey_wps_item
#endif
            ,&wps_set_context_plugin
            );

/*    WPS Settings MENU            */
/***********************************/


/***********************************/
/*    SETTINGS MENU                */

static struct browse_folder_info langs = { LANG_DIR, SHOW_LNG };

MENUITEM_FUNCTION_W_PARAM(browse_langs, 0, ID2P(LANG_LANGUAGE),
                          browse_folder, (void*)&langs, NULL, Icon_Language);

MAKE_MENU(settings_menu_item, ID2P(LANG_GENERAL_SETTINGS), 0,
          Icon_General_settings_menu,
          &wps_settings,
          &playlist_settings, &file_menu,
#ifdef HAVE_TAGCACHE
          &tagcache_menu,
#endif
          &display_menu, &system_menu,
          &startup_shutdown_menu,
          &bookmark_settings_menu,
#ifdef HAVE_TAGCACHE
          &autoresume_menu,
#endif
          &browse_langs, &voice_settings_menu,
          );
/*    SETTINGS MENU                */
/***********************************/
