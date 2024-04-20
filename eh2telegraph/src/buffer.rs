/// ImageBuffer for upload in batch. ceshi
pub struct ImageBuffer<T> {
    buf: Vec<T>,
    size: usize,
}

impl<T> Default for ImageBuffer<T> {
    #[inline]
    fn default() -> Self {
        Self {
            buf: Vec::new(),
            size: 0,
        }
    }
}

impl<T> ImageBuffer<T>
where
    T: DataSized,
{
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self {
            buf: Vec::with_capacity(n),
            size: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, data: T) {
        self.size += data.size();
        self.buf.push(data);
    }

    #[inline]
    pub fn swap(&mut self) -> (Vec<T>, usize) {
        let mut out = Vec::with_capacity(self.buf.len() * 2);
        std::mem::swap(&mut self.buf, &mut out);

        let mut size = 0;
        std::mem::swap(&mut self.size, &mut size);
        (out, size)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.len() == 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn clear(&mut self) {
        self.size = 0;
        self.buf.clear();
    }
}

pub trait DataSized {
    fn size(&self) -> usize;
}

impl DataSized for bytes::Bytes {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }
}

impl DataSized for Vec<u8> {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }
}

impl<const N: usize> DataSized for Box<[u8; N]> {
    #[inline]
    fn size(&self) -> usize {
        N
    }
}
