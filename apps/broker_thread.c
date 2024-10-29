/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2024 - Tsiry Sandratraina
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

bool broker_is_initialized = false;

/* Broker thread */
static long broker_stack[(DEFAULT_STACK_SIZE * 4)/sizeof(long)];
static const char broker_thread_name[] = "broker";
unsigned int broker_thread_id = 0;

extern void start_broker(void);

static void broker_thread(void) {
    start_broker();
}

/** -- Startup -- **/

/* Initialize the broker - called from init() in main.c */
void INIT_ATTR broker_init(void)
{
    /* Can never do this twice */
    if (broker_is_initialized)
    {
        logf("broker: already initialized");
        return;
    }

    logf("broker: initializing");

    /* Initialize queues before giving control elsewhere in case it likes
       to send messages. Thread creation will be delayed however so nothing
       starts running until ready if something yields such as talk_init. */
    // queue_init(&server_queue, true);
    broker_thread_id = create_thread(broker_thread, broker_stack,
                  sizeof(broker_stack), 0, broker_thread_name
                  IF_PRIO(,  PRIORITY_USER_INTERFACE)
                  IF_COP(, CPU));

    sleep(HZ); /* Give it a chance to start */

   /* Probably safe to say */
    broker_is_initialized = true;
}
