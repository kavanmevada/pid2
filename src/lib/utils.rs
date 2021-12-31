pub struct Dir<'de>(*mut libc::DIR, core::marker::PhantomData<&'de ()>,);

impl<'a> Dir<'a> {
    pub fn open(p: &str) -> Self {
        Self(sys!(opendir(c_str!(p))), Default::default())
    }
}

impl<'a> Iterator for Dir<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let dirent = sys!(readdir(self.0));
        let name = str!((*dirent).d_name.as_ptr());

        if dirent.is_null() { None }
        else if name == ".." || name == "." { self.next() }
        else { Some(name) }
    }
}