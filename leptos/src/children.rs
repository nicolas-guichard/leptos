use crate::into_view::{IntoView, View};
use std::sync::Arc;
use tachys::{
    renderer::dom::Dom,
    view::{
        any_view::{AnyView, IntoAny},
        RenderHtml,
    },
};

/// The most common type for the `children` property on components,
/// which can only be called once.
pub type Children = Box<dyn FnOnce() -> AnyView<Dom>>;

/// A type for the `children` property on components that can be called
/// more than once.
pub type ChildrenFn = Arc<dyn Fn() -> AnyView<Dom>>;

/// A type for the `children` property on components that can be called
/// more than once, but may mutate the children.
pub type ChildrenFnMut = Box<dyn FnMut() -> AnyView<Dom>>;

// This is to still support components that accept `Box<dyn Fn() -> AnyView>` as a children.
type BoxedChildrenFn = Box<dyn Fn() -> AnyView<Dom>>;

/// This trait can be used when constructing a component that takes children without needing
/// to know exactly what children type the component expects. This is used internally by the
/// `view!` macro implementation, and can also be used explicitly when using the builder syntax.
///
/// # Examples
///
/// ## Without ToChildren
///
/// Without [ToChildren], consumers need to explicitly provide children using the type expected
/// by the component. For example, [Provider][crate::Provider]'s children need to wrapped in
/// a [Box], while [Show][crate::Show]'s children need to be wrapped in an [Rc].
///
/// ```
/// # use leptos::{ProviderProps, ShowProps};
/// # use leptos_dom::html::p;
/// # use leptos_dom::IntoView;
/// # use leptos_macro::component;
/// # use std::rc::Rc;
/// #
/// #[component]
/// fn App() -> impl IntoView {
///     (
///         ProviderProps::builder()
///             .children(Box::new(|| p().child("Foo").into_view().into()))
///             // ...
/// #           .value("Foo")
/// #           .build(),
///         ShowProps::builder()
///             .children(Rc::new(|| p().child("Foo").into_view().into()))
///             // ...
/// #           .when(|| true)
/// #           .fallback(|| p().child("foo"))
/// #           .build(),
///     )
/// }
/// ```
///
/// ## With ToChildren
///
/// With [ToChildren], consumers don't need to know exactly which type a component uses for
/// its children.
///
/// ```
/// # use leptos::{ProviderProps, ShowProps};
/// # use leptos_dom::html::p;
/// # use leptos_dom::IntoView;
/// # use leptos_macro::component;
/// # use std::rc::Rc;
/// # use leptos::ToChildren;
/// #
/// #[component]
/// fn App() -> impl IntoView {
///     (
///         ProviderProps::builder()
///             .children(ToChildren::to_children(|| {
///                 p().child("Foo").into_view().into()
///             }))
///             // ...
/// #           .value("Foo")
/// #           .build(),
///         ShowProps::builder()
///             .children(ToChildren::to_children(|| {
///                 p().child("Foo").into_view().into()
///             }))
///             // ...
/// #           .when(|| true)
/// #           .fallback(|| p().child("foo"))
/// #           .build(),
///     )
/// }
pub trait ToChildren<F> {
    /// Convert the provided type to (generally a closure) to Self (generally a "children" type,
    /// e.g., [Children]). See the implementations to see exactly which input types are supported
    /// and which "children" type they are converted to.
    fn to_children(f: F) -> Self;
}

impl<F, C> ToChildren<F> for TypedChildren<C>
where
    F: FnOnce() -> C + 'static,
    C: IntoView,
{
    #[inline]
    fn to_children(f: F) -> Self {
        TypedChildren(Box::new(move || f().into_view()))
    }
}

impl<F, C> ToChildren<F> for Children
where
    F: FnOnce() -> C + 'static,
    C: RenderHtml<Dom> + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for ChildrenFn
where
    F: Fn() -> C + 'static,
    C: RenderHtml<Dom> + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for ChildrenFnMut
where
    F: Fn() -> C + 'static,
    C: RenderHtml<Dom> + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

impl<F, C> ToChildren<F> for BoxedChildrenFn
where
    F: Fn() -> C + 'static,
    C: RenderHtml<Dom> + 'static,
{
    #[inline]
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_any())
    }
}

/// New-type wrapper for the a function that returns a view with `From` and `Default` traits implemented
/// to enable optional props in for example `<Show>` and `<Suspense>`.
#[derive(Clone)]
pub struct ViewFn(Arc<dyn Fn() -> AnyView<Dom>>);

impl Default for ViewFn {
    fn default() -> Self {
        Self(Arc::new(|| ().into_any()))
    }
}

impl<F, C> From<F> for ViewFn
where
    F: Fn() -> C + 'static,
    C: RenderHtml<Dom> + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(move || value().into_any()))
    }
}

impl ViewFn {
    /// Execute the wrapped function
    pub fn run(&self) -> AnyView<Dom> {
        (self.0)()
    }
}

/// A typed equivalent to [`Children`], which takes a generic but preserves type information to
/// allow the compiler to optimize the view more effectively.
pub struct TypedChildren<T>(Box<dyn FnOnce() -> View<T>>);

impl<T> TypedChildren<T> {
    pub fn into_inner(self) -> impl FnOnce() -> View<T> {
        self.0
    }
}
