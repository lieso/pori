pub mod digest;

pub enum ContentPayload {
    Digest(digest::Digest),
}
