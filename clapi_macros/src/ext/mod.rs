
pub trait IteratorExt : Iterator{
    fn single(self) -> Option<Self::Item>
        where Self: Sized{
        let mut ret : Option<Self::Item> = None;

        for item in self {
            if ret.is_some(){
                return None;
            } else {
                ret = Some(item);
            }
        }

        ret
    }
}

impl<I: Iterator> IteratorExt for I{}