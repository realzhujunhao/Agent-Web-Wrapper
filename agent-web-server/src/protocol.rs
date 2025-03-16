use serde::{Serialize, ser::SerializeStruct};

/// The enum every API controller returns
///
/// A response can be  
/// either `{ success: true, data: {...} }`  
/// or     `{ success: false, err: {...} }`  
/// but never having both data and err.  
/// i.e. this is not possible  
/// ` { success: bool, data: {...}, err: {...} } `  
pub enum AppResp<T>
where
    T: Serialize,
{
    Success(T),
    Exception(String),
}

impl<T> Serialize for AppResp<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut struct_s = serializer.serialize_struct("AppResp", 2)?;
        match self {
            Self::Success(data) => {
                struct_s.serialize_field("success", &true)?;
                struct_s.serialize_field("data", data)?;
            }
            Self::Exception(err) => {
                struct_s.serialize_field("success", &false)?;
                struct_s.serialize_field("err", err)?;
            }
        }
        struct_s.end()
    }
}
