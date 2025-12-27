#ifndef HTTP_STREAM_H
#define HTTP_STREAM_H

/* http_stream.h
 *
 * Header for the Rust-implemented HTTP streaming backend.
 * This is the FFI boundary between Rockbox (C) and the Rust HTTP client.
 *
 * The functions below must be implemented in Rust as a static library
 * and exported with #[no_mangle] pub extern "C".
 */

#include <sys/types.h>  /* for off_t, ssize_t */

/* Opaque handle representing an open HTTP(S) connection + state */
struct http_stream_handle;

/**
 * Open an HTTP/HTTPS stream.
 *
 * @param url  Null-terminated URL string (e.g. "http://example.com/file.mp3")
 * @return     Pointer to opaque handle on success, NULL on failure
 */
struct http_stream_handle* http_stream_open(const char *url);

/**
 * Read data from the current position in the stream.
 *
 * @param handle  Handle returned by http_stream_open()
 * @param buf     Destination buffer
 * @param size    Number of bytes to read
 * @return        Number of bytes read (>=0), or -1 on error/end-of-stream
 */
ssize_t http_stream_read(struct http_stream_handle *handle,
                        void *buf, size_t size);

/**
 * Seek within the stream (uses HTTP Range requests when possible).
 *
 * @param handle  Handle returned by http_stream_open()
 * @param offset  Offset in bytes
 * @param whence  SEEK_SET (0), SEEK_CUR (1), or SEEK_END (2)
 * @return        New absolute position (>=0), or -1 on error
 */
off_t http_stream_lseek(struct http_stream_handle *handle,
                        off_t offset, int whence);

/**
 * Return the total size of the remote file, if known.
 *
 * This comes from Content-Length or Content-Range headers.
 * If the size is unknown (live stream, chunked encoding without length),
 * return -1.
 *
 * @param handle  Handle returned by http_stream_open()
 * @return        Total size in bytes, or -1 if unknown
 */
off_t http_stream_filesize(struct http_stream_handle *handle);

/**
 * Close the stream and free all associated resources (TCP connection, buffers, etc.).
 *
 * @param handle  Handle returned by http_stream_open()
 * @return        0 on success, -1 on error (errors can be ignored)
 */
int http_stream_close(struct http_stream_handle *handle);

#endif /* HTTP_STREAM_H */
