use std::ops::Deref;

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Some<T: 'static> {
    Owned(T),
    Static(&'static T),
    Signal(Signal<T>),
    Memo(Memo<T>),
    Mapped(MappedSignal<T>),
}

impl<T: 'static> From<T> for Some<T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<T: 'static> From<&'static T> for Some<T> {
    fn from(value: &'static T) -> Self {
        Self::Static(value)
    }
}

impl<T: 'static> From<Signal<T>> for Some<T> {
    fn from(value: Signal<T>) -> Self {
        Self::Signal(value)
    }
}

impl<T: 'static> From<Memo<T>> for Some<T> {
    fn from(value: Memo<T>) -> Self {
        Self::Memo(value)
    }
}

impl<T: 'static> From<MappedSignal<T>> for Some<T> {
    fn from(value: MappedSignal<T>) -> Self {
        Self::Mapped(value)
    }
}

impl<T: 'static + PartialEq> Some<T> {
    pub fn read(&self) -> impl Deref<Target = T> {
        match self {
            Self::Owned(data) => SomeRef::Owned(data),
            Self::Static(data) => SomeRef::Owned(*data),
            Self::Signal(data) => SomeRef::Signal(data.read()),
            Self::Memo(data) => SomeRef::Memo(data.read()),
            Self::Mapped(data) => SomeRef::Mapped(data.read()),
        }
    }
}

enum SomeRef<'a, T: 'static + PartialEq> {
    Owned(&'a T),
    Signal(ReadableRef<'a, Signal<T>>),
    Memo(ReadableRef<'a, Memo<T>>),
    Mapped(ReadableRef<'a, MappedSignal<T>>),
}

impl<T: PartialEq> Deref for SomeRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(r) => r,
            Self::Signal(r) => r.deref(),
            Self::Memo(r) => r.deref(),
            Self::Mapped(r) => r.deref(),
        }
    }
}
