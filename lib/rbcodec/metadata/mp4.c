/***************************************************************************
 *             __________               __   ___.
 *   Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
 *   Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
 *   Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
 *   Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
 *                     \/            \/     \/    \/            \/
 * $Id$
 *
 * Copyright (C) 2005 Magnus Holmgren
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
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <ctype.h>
#include <inttypes.h>
#include "platform.h"

#include "errno.h"
#include "metadata.h"
#include "metadata_common.h"
#include "metadata_parsers.h"
#include "logf.h"
#include "debug.h"
#include "replaygain.h"

#ifdef DEBUGF
#undef DEBUGF
#define DEBUGF(...)
#endif

#define MP4_3gp6 FOURCC('3', 'g', 'p', '6')
#define MP4_aART FOURCC('a', 'A', 'R', 'T')
#define MP4_alac FOURCC('a', 'l', 'a', 'c')
#define MP4_calb FOURCC(0xa9, 'a', 'l', 'b')
#define MP4_cART FOURCC(0xa9, 'A', 'R', 'T')
#define MP4_cgrp FOURCC(0xa9, 'g', 'r', 'p')
#define MP4_cgen FOURCC(0xa9, 'g', 'e', 'n')
#define MP4_chpl FOURCC('c', 'h', 'p', 'l')
#define MP4_cnam FOURCC(0xa9, 'n', 'a', 'm')
#define MP4_cwrt FOURCC(0xa9, 'w', 'r', 't')
#define MP4_ccmt FOURCC(0xa9, 'c', 'm', 't')
#define MP4_cday FOURCC(0xa9, 'd', 'a', 'y')
#define MP4_covr FOURCC('c', 'o', 'v', 'r')
#define MP4_disk FOURCC('d', 'i', 's', 'k')
#define MP4_esds FOURCC('e', 's', 'd', 's')
#define MP4_ftyp FOURCC('f', 't', 'y', 'p')
#define MP4_gnre FOURCC('g', 'n', 'r', 'e')
#define MP4_hdlr FOURCC('h', 'd', 'l', 'r')
#define MP4_ilst FOURCC('i', 'l', 's', 't')
#define MP4_mdat FOURCC('m', 'd', 'a', 't')
#define MP4_mdia FOURCC('m', 'd', 'i', 'a')
#define MP4_mdir FOURCC('m', 'd', 'i', 'r')
#define MP4_meta FOURCC('m', 'e', 't', 'a')
#define MP4_minf FOURCC('m', 'i', 'n', 'f')
#define MP4_moov FOURCC('m', 'o', 'o', 'v')
#define MP4_mp4a FOURCC('m', 'p', '4', 'a')
#define MP4_soun FOURCC('s', 'o', 'u', 'n')
#define MP4_stbl FOURCC('s', 't', 'b', 'l')
#define MP4_stsd FOURCC('s', 't', 's', 'd')
#define MP4_stts FOURCC('s', 't', 't', 's')
#define MP4_trak FOURCC('t', 'r', 'a', 'k')
#define MP4_trkn FOURCC('t', 'r', 'k', 'n')
#define MP4_udta FOURCC('u', 'd', 't', 'a')
#define MP4_extra FOURCC('-', '-', '-', '-')

/* Read the tag data from an MP4 file, storing up to buffer_size bytes in
 * buffer.
 */
static unsigned long read_mp4_tag(int fd, unsigned int size_left, char* buffer,
                                  unsigned int buffer_left)
{
    unsigned long bytes_read = 0;
    ssize_t rd_ret = 0;
    ssize_t bytes_req;
    #define MP4_TAG_HEADER_SIZE 16


    if (size_left >= MP4_TAG_HEADER_SIZE)
    {
        /* Skip the data tag header - maybe we should parse it properly? */
        lseek(fd, MP4_TAG_HEADER_SIZE, SEEK_CUR);
        size_left -= MP4_TAG_HEADER_SIZE;

        if (size_left > buffer_left)
            bytes_req = buffer_left;
        else
            bytes_req = size_left;

        rd_ret = read(fd, buffer, bytes_req);
        if (rd_ret == bytes_req)
            bytes_read = bytes_req;
        else
        {
            /* read less than expected or an error from read() */
            logf("Error %d, read_mp4_tag", rd_ret);
            if (rd_ret < 0)
                rd_ret = 0; /* Skip everything */
        }
    }
    if (size_left >  (unsigned int) rd_ret)
        lseek(fd, size_left - rd_ret, SEEK_CUR);


    return bytes_read;
}

/* Read a string tag from an MP4 file */
static unsigned int read_mp4_tag_string(int fd, int size_left, char** buffer,
                                        unsigned int* buffer_left, char** dest)
{
    unsigned int bytes_read = read_mp4_tag(fd, size_left, *buffer,
        *buffer_left > 0 ? *buffer_left - 1 : 0);
    unsigned int length = 0;

    if (bytes_read)
    {
        /* Do not overwrite already available metadata. Especially when reading
         * tags with e.g. multiple genres / artists. This way only the first 
         * of multiple entries is used, all following are dropped. */
        if (*dest == NULL)
        {
            (*buffer)[bytes_read] = 0; /* zero-terminate for correct strlen().*/
            length = strlen(*buffer) + 1;
            length = MIN(length, ID3V2_MAX_ITEM_SIZE); /* Limit item size. */

            *dest = *buffer;
            (*buffer)[length-1] = 0; /* zero-terminate buffer. */
            *buffer_left -= length;
            *buffer += length;
        }
    }
    else
    {
        *dest = NULL;
    }
    
    return length;
}

static unsigned int read_mp4_atom(int fd, uint32_t* size, 
                                  uint32_t* type, uint32_t size_left)
{
    read_uint32be(fd, size);
    read_uint32be(fd, type);

    if (*size == 1)
    {
        /* FAT32 doesn't support files this big, so something seems to 
         * be wrong. (64-bit sizes should only be used when required.)
         */
        errno = EFBIG;
        *type = 0;
        return 0;
    }

    if (*size > 0)
    {
        if (*size > size_left)
        {
            size_left = 0;
        }
        else
        {
            size_left -= *size;
        }
        
        *size -= 8;
    }
    else
    {
        *size = size_left;
        size_left = 0;
    }
    
    return size_left;
}

static unsigned int read_mp4_length(int fd, uint32_t* size)
{
    unsigned int length = 0;
    int bytes = 0;
    unsigned char c = '\0';

    do
    {
        read(fd, &c, 1);
        bytes++;
        (*size)--;
        length = (length << 7) | (c & 0x7F);
    }
    while ((c & 0x80) && (bytes < 4) && (*size > 0));

    return length;
}

static bool read_mp4_esds(int fd, struct mp3entry* id3, uint32_t* size)
{
    unsigned char buf[8] = {0};
    bool sbr = false;
    bool sbr_signaled = false;

    lseek(fd, 4, SEEK_CUR);     /* Version and flags. */
    read(fd, buf, 1);           /* Verify ES_DescrTag. */
    *size -= 5;

    if (*buf == 3)
    {
        /* read length */
        if (read_mp4_length(fd, size) < 20)
        {
            return sbr;
        }

        lseek(fd, 3, SEEK_CUR);
        *size -= 3;
    } 
    else
    {
        lseek(fd, 2, SEEK_CUR);
        *size -= 2;
    }

    read(fd, buf, 1);           /* Verify DecoderConfigDescrTab. */
    *size -= 1;

    if (*buf != 4)
    {
        return sbr;
    }

    if (read_mp4_length(fd, size) < 13)
    {
        return sbr;
    }
    
    lseek(fd, 13, SEEK_CUR);    /* Skip audio type, bit rates, etc. */
    read(fd, buf, 1);
    *size -= 14;
    
    if (*buf != 5)              /* Verify DecSpecificInfoTag. */
    {
        return sbr;
    }

    {
        static const int sample_rates[] =
        {
            96000, 88200, 64000, 48000, 44100, 32000,
            24000, 22050, 16000, 12000, 11025, 8000
        };
        unsigned long bits;
        unsigned int length;
        unsigned int index;
        unsigned int type;
        
        /* Read the (leading part of the) decoder config. */
        length = read_mp4_length(fd, size);
        length = MIN(length, *size);
        length = MIN(length, sizeof(buf));
        memset(buf, 0, sizeof(buf));
        read(fd, buf, length);
        *size -= length;
        
        /* Maybe time to write a simple read_bits function... */

        /* Decoder config format:
         * Object type           - 5 bits
         * Frequency index       - 4 bits
         * Channel configuration - 4 bits
         * Also see libfaad/mp4.c AudioSpecificConfig2 (consider using it instead of manual parsing)
         */
        bits = get_long_be(buf);
        type = bits >> 27;              /* Object type - 5 bits */
        index = (bits >> 23) & 0xf;     /* Frequency index - 4 bits */

        if (index < (sizeof(sample_rates) / sizeof(*sample_rates)))
        {
            id3->frequency = sample_rates[index];
        }
    
        if (type == 5)
        {
            unsigned int old_index = index;

            sbr = true;
            index = (bits >> 15) & 0xf; /* Frequency index - 4 bits */
    
            if (index == 15)
            {
                /* 17 bits read so far... */
                bits = get_long_be(&buf[2]);
                id3->frequency = (bits >> 7) & 0x00ffffff;
            }
            else if (index < (sizeof(sample_rates) / sizeof(*sample_rates)))
            {
                id3->frequency = sample_rates[index];
            }
            
            if (old_index == index)
            {
                /* Downsampled SBR */
                id3->frequency *= 2;
            }
        }
        /* Skip 13 bits from above, plus 3 bits, then read 11 bits */
        else if ((length >= 4) && (((bits >> 5) & 0x7ff) == 0x2b7))
        {
            /* We found an extensionAudioObjectType */
            type = bits & 0x1f;         /* Object type - 5 bits*/
            bits = get_long_be(&buf[4]);
            
            if (type == 5)
            {
                sbr = bits >> 31;
                sbr_signaled = true;

                if (sbr)
                {
                    unsigned int old_index = index;
                    
                    /* 1 bit read so far */
                    index = (bits >> 27) & 0xf; /* Frequency index - 4 bits */

                    if (index == 15)
                    {
                        /* 5 bits read so far */
                        id3->frequency = (bits >> 3) & 0x00ffffff;
                    }
                    else if (index < (sizeof(sample_rates) / sizeof(*sample_rates)))
                    {
                        id3->frequency = sample_rates[index];
                    }

                    if (old_index == index)
                    {
                        /* Downsampled SBR */
                        id3->frequency *= 2;
                    }
                }
            }
        }
#ifndef CODEC_AAC_SBR_DEC
        //SBR_DEC is disabled so disable sbr implicit signalling
        sbr_signaled = true;
#endif
        if (!sbr && !sbr_signaled && id3->frequency <= 24000)
        {
            /* As stated in libfaad/mp4.c AudioSpecificConfig2:
             * no SBR signalled, this could mean either implicit signalling or no SBR in this file 
             * MPEG specification states: assume SBR on files with samplerate <= 24000 Hz 
             */
            id3->frequency *= 2;
            sbr = true;
        }
    }
    
    return sbr;
}

static void read_mp4_tag_i_from_n(int fd, int *i, char** i_from_n_string, uint32_t size, unsigned int *buffer_left, char **buffer)
{
    uint16_t x[3];
    *i = 0;
    *i_from_n_string = NULL;
    if (read_mp4_tag(fd, size, (char*) &x, sizeof(x))  == sizeof(x))
    {
        *i = betoh16(x[1]);
        int n = betoh16(x[2]);
        if (n > 0)
        {
            unsigned int string_length = snprintf(*buffer, *buffer_left, "%d/%d", *i, n) + 1;
            if (string_length <= *buffer_left)
            {
                *i_from_n_string = *buffer;
                *buffer += string_length;
                *buffer_left -= string_length;
            }
        }
    }
}

static bool read_mp4_tags(int fd, struct mp3entry* id3, 
                          uint32_t size_left)
{
    uint32_t size;
    uint32_t type;
    unsigned int buffer_left = sizeof(id3->id3v2buf) + sizeof(id3->id3v1buf);
    char* buffer = id3->id3v2buf;
    bool cwrt = false;

    do
    {
        size_left = read_mp4_atom(fd, &size, &type, size_left);

        /* DEBUGF("Tag atom: '%c%c%c%c' (%d bytes left)\n", type >> 24 & 0xff, 
            type >> 16 & 0xff, type >> 8 & 0xff, type & 0xff, size); */

        switch (type)
        {
        case MP4_cnam:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left, 
                &id3->title);
            break;

        case MP4_cART:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left, 
                &id3->artist);
            break;

        case MP4_aART:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->albumartist);
            break;

        case MP4_cgrp:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->grouping);
            break;
        
        case MP4_calb:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->album);
            break;
        
        case MP4_cwrt:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->composer);
            cwrt = false;
            break;

        case MP4_ccmt:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->comment);
            break;

        case MP4_cday:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                                &id3->year_string);
 
            /* Try to parse it as a year, for the benefit of the database.
             */
            if(id3->year_string)
            {
                id3->year = atoi(id3->year_string);
                if (id3->year < 1900)
                {
                    id3->year = 0;
                }
            }
            else
                id3->year = 0;

            break;

        case MP4_gnre:
            {
                unsigned short genre;
                const unsigned int g_size = sizeof(genre);
                id3->genre_string = NULL;
                if (read_mp4_tag(fd, size, (char*) &genre, g_size) == g_size)
                    id3->genre_string = id3_get_num_genre(betoh16(genre) - 1);
            }
            break;
        
        case MP4_cgen:
            read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                &id3->genre_string);
            break;

        case MP4_disk:
            read_mp4_tag_i_from_n(fd, &id3->discnum, &id3->disc_string, size, &buffer_left, &buffer);
            break;

        case MP4_trkn:
            read_mp4_tag_i_from_n(fd, &id3->tracknum, &id3->track_string, size, &buffer_left, &buffer);
            break;

#ifdef HAVE_ALBUMART
        case MP4_covr:
            {
                int pos = lseek(fd, 0, SEEK_CUR) + 16;
                id3->albumart.type = AA_TYPE_UNKNOWN;
                
                if (read_mp4_tag(fd, size, buffer, 8) >= 4)
                {

                    if (memcmp(buffer, "\xff\xd8\xff\xe0", 4) == 0)
                    {
                        id3->albumart.type = AA_TYPE_JPG;
                    }
                    else if (memcmp(buffer, "\x89\x50\x4e\x47\x0d\x0a\x1a\x0a", 8) == 0)
                    {
                        id3->albumart.type = AA_TYPE_PNG;
                    }
                    
                    if (id3->albumart.type != AA_TYPE_UNKNOWN)
                    {
                        id3->albumart.pos  = pos;
                        id3->albumart.size = size - 16;
                        id3->has_embedded_albumart = true;
                    }
                }
            }
            break;
#endif

        case MP4_extra:
            {
                char tag_name[TAG_NAME_LENGTH];
                uint32_t sub_size;
                ssize_t rd_ret;
                /* "mean" atom */
                read_uint32be(fd, &sub_size);
                size -= sub_size;
                lseek(fd, sub_size - 4, SEEK_CUR);
                /* "name" atom */
                read_uint32be(fd, &sub_size);
                size -= sub_size;
                lseek(fd, 8, SEEK_CUR);
                sub_size -= 12;
                
                if (sub_size > sizeof(tag_name) - 1)
                {
                    rd_ret = read(fd, tag_name, sizeof(tag_name) - 1);                  
                    lseek(fd, sub_size - (sizeof(tag_name) - 1), SEEK_CUR);
                    sub_size = sizeof(tag_name) - 1;
                }
                else
                {
                    rd_ret = read(fd, tag_name, sub_size);
                }
                if (rd_ret != (ssize_t)sub_size)
                    rd_ret = 0;
                tag_name[rd_ret] = 0;

                static const char *tn_options[] = {"composer", "iTunSMPB",
                                   "musicbrainz track id", "album artist", NULL};

                int tn_op = string_option(tag_name, tn_options, true);


                if (tn_op == 0 && !cwrt) /*composer*/
                {
                    read_mp4_tag_string(fd, size, &buffer, &buffer_left, 
                        &id3->composer);
                }   
                else if (tn_op == 1) /*iTunSMPB*/
                {
                    char value[TAG_VALUE_LENGTH];
                    char* value_p = value;
                    char* any = NULL;
                    unsigned int length = sizeof(value);

                    read_mp4_tag_string(fd, size, &value_p, &length, &any);
                    id3->lead_trim = get_itunes_int32(value, 1);
                    id3->tail_trim = get_itunes_int32(value, 2);
                    DEBUGF("AAC: lead_trim %d, tail_trim %d\n", 
                        id3->lead_trim, id3->tail_trim);
                }
                else if (tn_op == 2) /*musicbrainz track id*/
                {
                    read_mp4_tag_string(fd, size, &buffer, &buffer_left,
                        &id3->mb_track_id);
                }
                else if (tn_op == 3) /*album artist*/
                {
                    read_mp4_tag_string(fd, size, &buffer, &buffer_left, 
                        &id3->albumartist);
                }   
                else
                {
                    char* any = NULL;
                    unsigned int length = read_mp4_tag_string(fd, size,
                        &buffer, &buffer_left, &any);

                    if (length > 0)
                    {
                        /* Re-use the read buffer as the dest buffer... */
                        buffer -= length;
                        buffer_left += length;
                        
                        parse_replaygain(tag_name, buffer, id3);
                    }
                }
            }
            break;
        
        default:
            lseek(fd, size, SEEK_CUR);
            break;
        }
    }
    while ((size_left > 0) && (errno == 0));

    return true;
}

static bool read_mp4_container(int fd, struct mp3entry* id3, 
                               uint32_t size_left)
{
    uint32_t size    = 0;
    uint32_t type    = 0;
    uint32_t handler = 0;
    bool rc = true;
    bool done = false;
    
    do
    {
        size_left = read_mp4_atom(fd, &size, &type, size_left);
        
        /* DEBUGF("Atom: '%c%c%c%c' (0x%08lx, %lu bytes left)\n", 
            (int) ((type >> 24) & 0xff), (int) ((type >> 16) & 0xff),
            (int) ((type >> 8) & 0xff), (int) (type & 0xff),
            type, size); */
        
        switch (type)
        {
        case MP4_ftyp:
            {
                // filetype (supported ignore case values: m4a, m4b, mp42, 3gp6, qt, isom)
                char filetype[4];
                read(fd, &filetype, 4);
                DEBUGF("MP4 file type:  '%.4s'\n", filetype);
                size -= 4;
            }
            break;

        case MP4_meta:
            lseek(fd, 4, SEEK_CUR);  /* Skip version */
            size -= 4;
            /* Fall through */

        case MP4_moov:
        case MP4_udta:
        case MP4_mdia:
        case MP4_stbl:
        case MP4_trak:
            rc = read_mp4_container(fd, id3, size);
            size = 0;
            break;
        
        case MP4_ilst:
            /* We need at least a size of 8 to read the next atom. */
            if (handler == MP4_mdir && size>8)
            {
                rc = read_mp4_tags(fd, id3, size);
                size = 0;
            }
            break;
        
        case MP4_minf:
            if (handler == MP4_soun)
            {
                rc = read_mp4_container(fd, id3, size);
                size = 0;
            }
            break;
        
        case MP4_stsd:
            lseek(fd, 8, SEEK_CUR);
            size -= 8;
            rc = read_mp4_container(fd, id3, size);
            size = 0;
            break;
        
        case MP4_hdlr:
            lseek(fd, 8, SEEK_CUR);
            read_uint32be(fd, &handler);
            size -= 12;
            /* DEBUGF("    Handler '%c%c%c%c'\n", handler >> 24 & 0xff, 
                handler >> 16 & 0xff, handler >> 8 & 0xff,handler & 0xff); */
            break;
        
        case MP4_stts:
            {
                uint32_t entries;
                unsigned int i;

                /* Reset to false. */
                id3->needs_upsampling_correction = false;

                lseek(fd, 4, SEEK_CUR);
                read_uint32be(fd, &entries);
                id3->samples = 0;

                for (i = 0; i < entries; i++)
                {
                    uint32_t n;
                    uint32_t l;

                    read_uint32be(fd, &n);
                    read_uint32be(fd, &l);

                    /* Some AAC file use HE profile. In this case the number
                     * of output samples is doubled to a maximum of 2048 
                     * samples per frame. This means that files which already 
                     * report a frame size of 2048 in their header will not 
                     * need any further special handling. */
                    if (id3->codectype==AFMT_MP4_AAC_HE && l<=1024)
                    {
                        l *= 2;
                        id3->needs_upsampling_correction = true;
                    }

                    id3->samples += (uint64_t) n * l;
                }
                
                size = 0;
            }
            break;

        case MP4_mp4a:
            {
                uint32_t subsize;
                uint32_t subtype;

                /* Move to the next expected mp4 atom. */
                lseek(fd, 28, SEEK_CUR);
                read_mp4_atom(fd, &subsize, &subtype, size);
                size -= 36;

                if (subtype == MP4_esds)
                {
                    /* Read esds metadata and return if AAC-HE/SBR is used. */
                    if (read_mp4_esds(fd, id3, &size))
                        id3->codectype = AFMT_MP4_AAC_HE;
                    else
                        id3->codectype = AFMT_MP4_AAC;
                }
            }
            break;

        case MP4_alac:
            {
                uint32_t frequency;
                uint32_t subsize;
                uint32_t subtype;

                /* Move to the next expected mp4 atom. */
                lseek(fd, 28, SEEK_CUR);
                read_mp4_atom(fd, &subsize, &subtype, size);
                size -= 36;
#if 0
                /* We might need to parse for the alac metadata atom. */
                while (!((subsize==28) && (subtype==MP4_alac)) && (size>0))
                {
                    lseek(fd, -7, SEEK_CUR);
                    read_mp4_atom(fd, &subsize, &subtype, size);
                    size -= 1;
                    errno = 0; /* will most likely be set while parsing */
                }
#endif
                if (subtype == MP4_alac)
                {
                    lseek(fd, 24, SEEK_CUR);
                    read_uint32be(fd, &frequency);
                    size -= 28;
                    id3->frequency = frequency;
                    id3->codectype = AFMT_MP4_ALAC;
                }
            }
            break;

        case MP4_mdat:
            /* Some AAC files appear to contain additional empty mdat chunks.
               Ignore them. */
            if(size == 0)
                break;
            /* mdat chunks accumulate! */
            id3->filesize += size;
            if(id3->samples > 0) {
                /* We've already seen the moov chunk. */
                done = true;
            }
            break;

        case MP4_chpl:
            {
                /* ADDME: add support for real chapters. Right now it's only
                 * used for Nero's gapless hack */
                uint8_t chapters   = 0;
                uint64_t timestamp = 0;

                lseek(fd, 8, SEEK_CUR);
                read_uint8(fd, &chapters);
                size -= 9;

                /* the first chapter will be used as the lead_trim */
                if (chapters > 0) {
                    read_uint64be(fd, &timestamp);
                    id3->lead_trim = (timestamp * id3->frequency) / 10000000;
                    size -= 8;
                }
            }
            break;

        default:
            break;
        }
        
        /* Skip final seek. */
        if (!done)
        {
            lseek(fd, size, SEEK_CUR);
        }
    } while (rc && (size_left > 0) && (errno == 0) && !done);
    
    return rc;
}

bool get_mp4_metadata(int fd, struct mp3entry* id3)
{
    id3->codectype = AFMT_UNKNOWN;
    id3->filesize = 0;
    errno = 0;

    if (read_mp4_container(fd, id3, filesize(fd)) && (errno == 0) 
        && (id3->samples > 0) && (id3->frequency > 0) 
        && (id3->filesize > 0))
    {
        if (id3->codectype == AFMT_UNKNOWN)
        {
            logf("Not an ALAC or AAC file");
            return false;
        }

        id3->length = ((int64_t) id3->samples * 1000) / id3->frequency;

        id3->vbr = true; /* ALAC is native VBR, AAC very unlikely is CBR. */

        if (id3->length <= 0)
        {
            logf("mp4 length invalid!");
            return false;
        }

        id3->bitrate = ((int64_t) id3->filesize * 8) / id3->length;
        DEBUGF("MP4 bitrate %d, frequency %ld Hz, length %ld ms\n",
            id3->bitrate, id3->frequency, id3->length);
    }
    else
    {
        logf("MP4 metadata error");
        DEBUGF("MP4 metadata error. errno %d, samples %ld, frequency %ld, "
            "filesize %ld\n", errno, id3->samples, id3->frequency,
            id3->filesize);
        return false;
    }

    return true;
}
