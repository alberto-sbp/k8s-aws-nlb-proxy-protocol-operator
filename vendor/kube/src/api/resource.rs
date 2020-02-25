#![allow(non_snake_case)]

use crate::api::metadata::{ListMeta, ObjectMeta, TypeMeta};
use crate::ErrorResponse;
use serde::Deserialize;
use std::fmt::Debug;

/// Accessor trait needed to build higher level abstractions on kubernetes objects
pub trait KubeObject {
    /// Every object must have ObjectMeta
    fn meta(&self) -> &ObjectMeta;
    // NB: TypeMeta also required, but not used by abstractions yet
}

/// A raw event returned from a watch query
///
/// Note that a watch query returns many of these as newline separated json.
#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "object", rename_all = "UPPERCASE")]
pub enum WatchEvent<K>
where
    K: Clone + KubeObject,
{
    Added(K),
    Modified(K),
    Deleted(K),
    Error(ErrorResponse),
}

impl<K> Debug for WatchEvent<K>
where
    K: Clone + KubeObject,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            WatchEvent::Added(_) => write!(f, "Added event"),
            WatchEvent::Modified(_) => write!(f, "Modified event"),
            WatchEvent::Deleted(_) => write!(f, "Deleted event"),
            WatchEvent::Error(e) => write!(f, "Error event: {:?}", e),
        }
    }
}

// -------------------------------------------------------

/// A standard kubernetes object with .spec and .status
///
/// This is used instead of a full struct for `Deployment`, `Pod`, `Node`, `CRD`, ...
/// Kubernetes' API generally exposes core structs in this manner, but sometimes the
/// status, `U`, is missing, and is therefore wrapped in `Option`.
///
/// The reasons we use this wrapper rather than the actual structs are:
/// - metadata field requirement for generic Informers is impossible (no field level traits)
/// - you cannot implement traits for objects you don't own => no addon traits to k8s-openapi
///
/// This struct appears in `ObjectList` and `WatchEvent`, and when using a `Reflector`,
/// and is exposed as the values in `ObjectMap`.
#[derive(Deserialize, Serialize, Clone)]
pub struct Object<P, U>
where
    P: Clone,
    U: Clone,
{
    #[serde(flatten)]
    pub types: TypeMeta,

    /// Resource metadata
    ///
    /// Contains information common to most resources about the Resource,
    /// including the object name, annotations, labels and more.
    pub metadata: ObjectMeta,

    /// The Spec struct of a resource. I.e. `PodSpec`, `DeploymentSpec`, etc.
    ///
    /// This defines the desired state of the Resource as specified by the user.
    pub spec: P,

    /// The Status of a resource. I.e. `PotStatus`, `DeploymentStatus`, etc.
    ///
    /// This publishes the state of the Resource as observed by the controller.
    /// Internally passed as `Option<()>` when a status does not exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<U>,
}

/// Blanked implementation for standard objects that can use Object
impl<P, U> KubeObject for Object<P, U>
where
    P: Clone,
    U: Clone,
{
    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }
}

/// A generic kubernetes object list
///
/// This is used instead of a full struct for `DeploymentList`, `PodList`, etc.
/// Kubernetes' API [always seem to expose list structs in this manner](https://docs.rs/k8s-openapi/0.4.0/k8s_openapi/apimachinery/pkg/apis/meta/v1/struct.ObjectMeta.html?search=List).
///
/// Note that this is only used internally within reflectors and informers,
/// and is generally produced from list/watch/delete collection queries on an `RawApi`.
#[derive(Deserialize)]
pub struct ObjectList<T>
where
    T: Clone,
{
    // NB: kind and apiVersion can be set here, but no need for it atm
    /// ListMeta - only really used for its resourceVersion
    ///
    /// See [ListMeta](https://docs.rs/k8s-openapi/0.4.0/k8s_openapi/apimachinery/pkg/apis/meta/v1/struct.ListMeta.html)
    pub metadata: ListMeta,

    /// The items we are actually interested in. In practice; T:= Resource<T,U>.
    #[serde(bound(deserialize = "Vec<T>: Deserialize<'de>"))]
    pub items: Vec<T>,
}

impl<T: Clone> ObjectList<T> {
    /// `iter` returns an Iterator over the elements of this ObjectList
    ///
    /// # Example
    ///
    /// ```
    /// use kube::api::{ListMeta, ObjectList};
    ///
    /// let metadata: ListMeta = Default::default();
    /// let items = vec![1, 2, 3];
    /// let objectlist = ObjectList { metadata, items };
    ///
    /// let first = objectlist.iter().next();
    /// println!("First element: {:?}", first); // prints "First element: Some(1)"
    /// ```
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.items.iter()
    }

    /// `iter_mut` returns an Iterator of mutable references to the elements of this ObjectList
    ///
    /// # Example
    ///     
    /// ```
    /// use kube::api::{ObjectList, ListMeta};
    ///
    /// let metadata: ListMeta = Default::default();
    /// let items = vec![1, 2, 3];
    /// let mut objectlist = ObjectList { metadata, items };
    ///
    /// let mut first = objectlist.iter_mut().next();
    ///
    /// // Reassign the value in first
    /// if let Some(elem) = first {
    ///     *elem = 2;
    ///     println!("First element: {:?}", elem); // prints "First element: 2"
    /// }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut T> + 'a {
        self.items.iter_mut()
    }
}

impl<T: Clone> IntoIterator for ObjectList<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a ObjectList<T> {
    type Item = &'a T;
    type IntoIter = ::std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut ObjectList<T> {
    type Item = &'a mut T;
    type IntoIter = ::std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}
