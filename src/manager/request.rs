#[derive(FromForm)]
pub struct AddRequest<'r> {
    #[field(name = "resource")]
    pub resource_url: &'r str,

    #[field(name = "resource_proxy")]
    pub proxy_resource_url: &'r str,

    #[field(name = "resource_content_type")]
    pub resource_content_type: &'r str
}

#[derive(FromForm)]
pub struct RemoveRequest<'r> {
    #[field(name = "resource")]
    pub resource_url: &'r str
}

#[derive(FromForm)]
pub struct ModifyRequest<'r> {
    #[field(name = "resource")]
    pub resource_url: &'r str,

    #[field(name = "resource_proxy")]
    pub proxy_resource_url: &'r str,

    #[field(name = "resource_content_type")]
    pub resource_content_type: &'r str
}

#[derive(FromForm)]
pub struct SetProxyEnabledRequest {
    #[field(name = "enabled")]
    pub enabled: bool,
}
