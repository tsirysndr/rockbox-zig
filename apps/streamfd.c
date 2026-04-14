/***************************************************************************
 * Unified stream I/O dispatch for local files and HTTP(S) network streams.
 * Active in the SDL simulator build and the hosted SDL application build.
 *
 * See streamfd.h for the public interface and fd-encoding documentation.
 ***************************************************************************/

#include "config.h"

#if defined(SIMULATOR) || defined(APPLICATION)

#include "streamfd.h"
#include "file.h"       /* for app_open / app_read / app_lseek / app_close /
                           app_filesize and sim_* equivalents (via macros) */
#include <string.h>
#include <stdint.h>
#include <fcntl.h>

/* ------------------------------------------------------------------
 * C declarations for the Rust ABI exported by crates/netstream.
 * ------------------------------------------------------------------ */
extern int32_t  rb_net_open  (const char *url);
extern int64_t  rb_net_read  (int32_t h, void *dst,   size_t n);
extern int64_t  rb_net_lseek (int32_t h, int64_t off, int32_t whence);
extern int64_t  rb_net_len   (int32_t h);
extern void     rb_net_close (int32_t h);

/* ------------------------------------------------------------------ */

/** Convert an HTTP fd (<=STREAM_HTTP_FD_BASE) back to a Rust handle id. */
static inline int32_t http_fd_to_handle(int fd)
{
    return (int32_t)(STREAM_HTTP_FD_BASE - fd);
}

/* ------------------------------------------------------------------ */

static int path_is_url(const char *path)
{
    return (strncmp(path, "http://",  7) == 0 ||
            strncmp(path, "https://", 8) == 0);
}

int stream_open(const char *path, int flags)
{
    if (path == NULL)
        return -1;

    if (path_is_url(path)) {
        int32_t h = rb_net_open(path);
        if (h < 0)
            return -1;
        /* Map handle 0 -> -1000, handle 1 -> -1001, etc. */
        return STREAM_HTTP_FD_BASE - (int)h;
    }

    return open(path, flags);
}

ssize_t stream_read(int fd, void *buf, size_t n)
{
    if (stream_is_http_fd(fd)) {
        int64_t r = rb_net_read(http_fd_to_handle(fd), buf, n);
        return (ssize_t)r;
    }
    return read(fd, buf, n);
}

off_t stream_lseek(int fd, off_t off, int whence)
{
    if (stream_is_http_fd(fd)) {
        int64_t r = rb_net_lseek(http_fd_to_handle(fd), (int64_t)off, whence);
        return (off_t)r;
    }
    return lseek(fd, off, whence);
}

int stream_close(int fd)
{
    if (fd == -1)
        return 0;
    if (stream_is_http_fd(fd)) {
        rb_net_close(http_fd_to_handle(fd));
        return 0;
    }
    return close(fd);
}

off_t stream_filesize_fd(int fd)
{
    if (stream_is_http_fd(fd)) {
        int64_t len = rb_net_len(http_fd_to_handle(fd));
        if (len < 0) {
            /*
             * Content-Length unknown: return a large sentinel (~2 GiB).
             * The buffering layer truncates h->filesize to h->end when
             * read() returns 0 (EOF), so this is safe for finite streams.
             */
            return (off_t)0x7FFFFFFF;
        }
        return (off_t)len;
    }
    return filesize(fd);
}

#endif /* SIMULATOR || APPLICATION */
