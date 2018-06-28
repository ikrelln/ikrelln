use std::str::FromStr;

#[derive(Debug)]
pub struct KnownTag {
    pub tag: String,
}
impl From<OpenTracingTag> for KnownTag {
    fn from(tag: OpenTracingTag) -> KnownTag {
        let tag_str: &'static str = tag.into();
        KnownTag {
            tag: tag_str.to_string(),
        }
    }
}
impl From<IkrellnTags> for KnownTag {
    fn from(tag: IkrellnTags) -> KnownTag {
        let tag_str: &'static str = tag.into();
        KnownTag {
            tag: tag_str.to_string(),
        }
    }
}

// OpenTracing semantics v1.1
// https://github.com/opentracing/specification/blob/master/semantic_conventions.md#span-tags-table
#[derive(Clone)]
pub enum OpenTracingTag {
    Component,
    DbInstance,
    DbStatement,
    DbType,
    DbUser,
    Error,
    HttpMethod,
    HttpStatusCode,
    HttpUrl,
    MessageBusDestination,
    PeerAddress,
    PeerHostname,
    PeerIpv4,
    PeerIpv6,
    PeerPort,
    PeerService,
    SamplingPriority,
    SpanKind,
}
impl From<OpenTracingTag> for &'static str {
    fn from(tag: OpenTracingTag) -> &'static str {
        match tag {
            OpenTracingTag::Component => "component",
            OpenTracingTag::DbInstance => "db.instance",
            OpenTracingTag::DbStatement => "db.statement",
            OpenTracingTag::DbType => "db.type",
            OpenTracingTag::DbUser => "db.user",
            OpenTracingTag::Error => "error",
            OpenTracingTag::HttpMethod => "http.method",
            OpenTracingTag::HttpStatusCode => "http.status_code",
            OpenTracingTag::HttpUrl => "http.url",
            OpenTracingTag::MessageBusDestination => "message_bus.destination",
            OpenTracingTag::PeerAddress => "peer.address",
            OpenTracingTag::PeerHostname => "peer.hostname",
            OpenTracingTag::PeerIpv4 => "peer.ipv4",
            OpenTracingTag::PeerIpv6 => "peer.ipv6",
            OpenTracingTag::PeerPort => "peer.port",
            OpenTracingTag::PeerService => "peer.service",
            OpenTracingTag::SamplingPriority => "sampling.priority",
            OpenTracingTag::SpanKind => "span.kind",
        }
    }
}
pub struct NonOpenTracingTag;
impl FromStr for OpenTracingTag {
    type Err = NonOpenTracingTag;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "component" => Ok(OpenTracingTag::Component),
            "db.instance" => Ok(OpenTracingTag::DbInstance),
            "db.statement" => Ok(OpenTracingTag::DbStatement),
            "db.type" => Ok(OpenTracingTag::DbType),
            "db.user" => Ok(OpenTracingTag::DbUser),
            "error" => Ok(OpenTracingTag::Error),
            "http.method" => Ok(OpenTracingTag::HttpMethod),
            "http.status_code" => Ok(OpenTracingTag::HttpStatusCode),
            "http.url" => Ok(OpenTracingTag::HttpUrl),
            "message_bus.destination" => Ok(OpenTracingTag::MessageBusDestination),
            "peer.address" => Ok(OpenTracingTag::PeerAddress),
            "peer.hostname" => Ok(OpenTracingTag::PeerHostname),
            "peer.ipv4" => Ok(OpenTracingTag::PeerIpv4),
            "peer.ipv6" => Ok(OpenTracingTag::PeerIpv6),
            "peer.port" => Ok(OpenTracingTag::PeerPort),
            "peer.service" => Ok(OpenTracingTag::PeerService),
            "sampling.priority" => Ok(OpenTracingTag::SamplingPriority),
            "span.kind" => Ok(OpenTracingTag::SpanKind),
            &_ => Err(NonOpenTracingTag),
        }
    }
}

#[derive(Clone)]
pub enum IkrellnTags {
    Class,
    Environment,
    Name,
    Result,
    StepParameters,
    StepStatus,
    StepType,
    Suite,
}
impl From<IkrellnTags> for &'static str {
    fn from(tag: IkrellnTags) -> &'static str {
        match tag {
            IkrellnTags::Class => "test.class",
            IkrellnTags::Environment => "test.environment",
            IkrellnTags::Name => "test.name",
            IkrellnTags::Result => "test.result",
            IkrellnTags::StepParameters => "test.step_parameters",
            IkrellnTags::StepStatus => "test.step_status",
            IkrellnTags::StepType => "test.step_type",
            IkrellnTags::Suite => "test.suite",
        }
    }
}
pub struct NonIkrellnTag;
impl FromStr for IkrellnTags {
    type Err = NonIkrellnTag;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test.class" => Ok(IkrellnTags::Class),
            "test.environment" => Ok(IkrellnTags::Environment),
            "test.name" => Ok(IkrellnTags::Name),
            "test.result" => Ok(IkrellnTags::Result),
            "test.step_parameters" => Ok(IkrellnTags::StepParameters),
            "test.step_status" => Ok(IkrellnTags::StepStatus),
            "test.step_type" => Ok(IkrellnTags::StepType),
            "test.suite" => Ok(IkrellnTags::Suite),
            &_ => Err(NonIkrellnTag),
        }
    }
}
