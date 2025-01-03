pub struct ScrapeJob {
    pub query: String,
    pub store: Option<Store>,
    pub priority: JobPriority,
    pub num_products: Option<usize>,
} 