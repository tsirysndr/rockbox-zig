/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2005-2007 Miika Pekkarinen
 * Copyright (C) 2007-2008 Nicolas Pennequin
 * Copyright (C) 2011-2013 Michael Sevakis
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
#include "config.h"
#include "system.h"
#include "kernel.h"
#include "logf.h"
#include "appevents.h"

bool server_is_initialized = false;

/* Event queues */
// struct event_queue server_queue SHAREDBSS_ATTR;
// static struct queue_sender_list server_queue_sender_list SHAREDBSS_ATTR;

/* Server thread */
static long server_stack[(DEFAULT_STACK_SIZE + 0x1000)/sizeof(long)];
static const char server_thread_name[] = "server";
unsigned int server_thread_id = 0;

extern void start_server(void);
extern void start_servers(void);

extern void debugfn(const char *fmt);

static void server_thread(void) {
    start_server();
}

/** -- Startup -- **/

/* Initialize the server - called from init() in main.c */
void INIT_ATTR server_init(void)
{
    /* Can never do this twice */
    if (server_is_initialized)
    {
        logf("server: already initialized");
        return;
    }

    logf("server: initializing");

    /* Initialize queues before giving control elsewhere in case it likes
       to send messages. Thread creation will be delayed however so nothing
       starts running until ready if something yields such as talk_init. */
    // queue_init(&server_queue, true);
    server_thread_id = create_thread(server_thread, server_stack,
                  sizeof(server_stack), 0, server_thread_name
                  IF_PRIO(,  PRIORITY_USER_INTERFACE)
                  IF_COP(, CPU));

    sleep(HZ); /* Give it a chance to start */
    
    start_servers();

   /* Probably safe to say */
    server_is_initialized = true;
}
