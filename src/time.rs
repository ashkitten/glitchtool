use futures::stream::{BoxStream, StreamExt};
use iced::{futures, Subscription};
use iced_native::subscription::Recipe;
use std::{
    hash::{Hash, Hasher},
    time::Duration,
};

pub fn every(duration: Duration) -> Subscription<()> {
    Subscription::from_recipe(Every(duration))
}

struct Every(Duration);

impl<H, I> Recipe<H, I> for Every
where
    H: Hasher,
{
    type Output = ();

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<'static, I>) -> BoxStream<'static, Self::Output> {
        async_std::stream::interval(self.0).boxed()
    }
}
