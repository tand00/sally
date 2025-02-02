use std::sync::Arc;

use crate::translation::observation::observable::Observable;

pub struct BeliefsNode<T : Observable> {
    pub observation : Arc<T::Observed>,
    pub possibilities : Vec<Arc<T>>
}

pub struct BeliefsGraph<T : Observable> {
    pub nodes : Vec<Arc<BeliefsNode<T>>>
}