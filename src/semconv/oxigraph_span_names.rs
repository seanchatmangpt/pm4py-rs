/// Oxigraph query — runs SPARQL SELECT, ASK, or CONSTRUCT against the /query endpoint.
///
/// Span: `span.oxigraph.query`
/// Kind: `client`
/// Stability: `development`
pub const OXIGRAPH_QUERY_SPAN: &str = "oxigraph.query";
/// Oxigraph write — loads Turtle or N-Triples RDF data into Oxigraph via HTTP POST /store.
///
/// Span: `span.oxigraph.write`
/// Kind: `client`
/// Stability: `development`
pub const OXIGRAPH_WRITE_SPAN: &str = "oxigraph.write";
