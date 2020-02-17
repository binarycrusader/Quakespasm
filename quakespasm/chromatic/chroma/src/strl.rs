/*
 * Copyright (c) 1998 Todd C. Miller <Todd.Miller@courtesan.com>
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
 * OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */

pub mod capi {
    use libc::{c_char, size_t};
    use std::slice;

    /*
     * Copy src to string dst of size siz.  At most siz-1 characters
     * will be copied.  Always NUL terminates (unless siz == 0).
     * Returns strlen(src); if retval >= siz, truncation occurred.
     */
    #[no_mangle]
    pub extern "C" fn q_strlcpy(dst: *mut c_char, src: *const c_char, siz: size_t) -> size_t {
        let src_strlen = unsafe { libc::strlen(src) };
        let src_slice = unsafe { slice::from_raw_parts(src, src_strlen + 1) };

        if siz != 0 {
            let dst_slice = unsafe { slice::from_raw_parts_mut(dst, siz) };

            let max_len = (siz - 1).min(src_strlen);
            let (src_bytes, _) = src_slice.split_at(max_len);
            let (dst_bytes, dst_nul_bytes) = dst_slice.split_at_mut(max_len);
            dst_bytes.copy_from_slice(src_bytes);
            dst_nul_bytes[0] = '\0' as i8;
        }

        return src_strlen;
    }

    /*
     * Appends src to string dst of size siz (unlike strncat, siz is the
     * full size of dst, not space left).  At most siz-1 characters
     * will be copied.  Always NUL terminates (unless siz <= strlen(dst)).
     * Returns strlen(src) + MIN(siz, strlen(initial dst)).
     * If retval >= siz, truncation occurred.
     */
    #[no_mangle]
    pub extern "C" fn q_strlcat(dst: *mut c_char, src: *const c_char, siz: size_t) -> size_t {
        let dst_slice = unsafe { slice::from_raw_parts_mut(dst, siz) };

        let dst_strlen = dst_slice.iter().position(|&c| c == 0).unwrap_or(siz);
        if siz <= dst_strlen {
            return unsafe { libc::strlen(src) } + dst_strlen;
        }

        let (_, dst_rem) = dst_slice.split_at_mut(dst_strlen);
        return q_strlcpy(dst_rem.as_mut_ptr(), src, dst_rem.len()) + dst_strlen;
    }
}
