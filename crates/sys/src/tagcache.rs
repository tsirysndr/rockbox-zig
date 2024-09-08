pub fn search() {
    unsafe {
        crate::tagcache_search();
    }
}

pub fn search_set_uniqbuf() {
    unsafe {
        crate::tagcache_search_set_uniqbuf();
    }
}

pub fn search_add_filter() {
    unsafe {
        crate::tagcache_search_add_filter();
    }
}

pub fn get_next() {
    unsafe {
        crate::tagcache_get_next();
    }
}

pub fn get_numeric() {
    unsafe {
        crate::tagcache_get_numeric();
    }
}

pub fn get_stat() {
    unsafe {
        crate::tagcache_get_stat();
    }
}

pub fn commit_finalize() {
    unsafe {
        crate::tagcache_commit_finalize();
    }
}

pub fn tagtree_subentries_do_action() {
    unsafe {
        crate::tagtree_subentries_do_action();
    }
}

pub fn search_albumart_files() {
    unsafe {
        crate::search_albumart_files();
    }
}
