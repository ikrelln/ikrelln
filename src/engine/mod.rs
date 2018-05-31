use serde;
use std::fmt;
use uuid;

pub mod ingestor;
pub mod report;
pub mod span;
pub mod streams;
pub mod test_result;

pub fn hello() -> &'static str {
    "I am i'Krelln"
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct $name(pub String);
        impl TypedId for $name {}
        impl From<String> for $name {
            fn from(v: String) -> Self {
                $name(v)
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
        impl $name {
            #[allow(dead_code)]
            pub fn new() -> $name {
                $name(format!("{}", uuid::Uuid::new_v4().hyphenated()))
            }
        }
    };
}

trait TypedId {}

typed_id!(ApplicationId);
typed_id!(TestId);
typed_id!(StepId);
typed_id!(TagId);

typed_id!(IngestId);

trait HasId<S> {
    fn id(&self) -> &S
    where
        S: TypedId;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Application {
    pub id: ApplicationId,
    pub name: String,
    pub tags: Vec<Tag>,
}
impl HasId<ApplicationId> for Application {
    fn id(&self) -> &ApplicationId {
        &self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Test {
    pub id: TestId,
    #[serde(serialize_with = "serialize_with_id", rename = "application_id")]
    pub application: Application,
    pub name: String,
    pub duration: u64,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Step {
    pub id: StepId,
    pub test: Test,
    #[serde(serialize_with = "serialize_option_with_id", rename = "application_id")]
    pub application: Option<Application>,
    pub name: String,
    pub duration: u64,
    pub tags: Vec<Tag>,
}
impl HasId<StepId> for Step {
    fn id(&self) -> &StepId {
        &self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
}

fn serialize_with_id<S, T, U>(x: &T, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: HasId<U>,
    U: TypedId + serde::Serialize,
{
    serde::Serialize::serialize(&x.id(), serializer)
}

fn serialize_option_with_id<S, T, U>(
    x: &Option<T>,
    serializer: S,
) -> ::std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: HasId<U>,
    U: TypedId + serde::Serialize,
{
    if let Some(ref y) = *x {
        serde::Serialize::serialize(&y.id(), serializer)
    } else {
        serde::Serialize::serialize::<S>(&(None as Option<U>), serializer)
    }
}
