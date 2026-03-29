# Author Information

## Primary Author

### Sean Chatman
**Title:** Founder & Staff Engineer, ChatmanGPT
**Education:** Self-directed learner with 25+ years of professional software engineering experience

**Professional Background:**
- **Current:** Contracting at Disney Studios (Frontend Engineer, AI-assisted development task force)
- **Earlier:** Positions in fintech (Intuit), entertainment (Method Studios, Disney), telecom (AT&T), gaming
- **Expertise:** Python, TypeScript, JavaScript (expert); Java, Rust, Erlang, Go (production experience)
- **Process Mining:** Creator of dspygen (130+ GitHub stars), YAWL v6 (Java 25 workflow engine), 15+ process mining projects

**Key Publications & Open Source:**
- **dspygen:** DSP (Domain-Specific Programming) framework; 130+ GitHub stars, 23 forks
- **YAWL v6:** Complete Yet Another Workflow Language implementation in Java 25
- **ChatmanGPT Research:** Signal Theory (S=(M,G,T,F,W)), Chatman Equation (A=μ(O)), 7-Layer Architecture
- **GitHub:** https://github.com/seanchatmangpt
- **Statistics:** 4,787 contributions in 2025 alone

**Research Interests:**
- Process mining and business process optimization
- Formal verification and type system applications in numerical computing
- Workflow languages and autonomous system orchestration
- AI-assisted software development and augmented engineering

**Contact Information:**
- Email: info@chatmangpt.com
- Phone: 323-252-2071
- Address: 115 E Del Mar Blvd, Unit 405, Pasadena, CA 91105
- GitHub: https://github.com/seanchatmangpt
- LinkedIn: https://linkedin.com/in/seanchatman

---

## Contributions Summary

### Sean Chatman

**PM4Py-Rust Implementation:**
- Core library architecture and module design
- All 8 discovery algorithms (Alpha, Inductive, Heuristic, DFG, ILP, Split, Causal Net, Tree miners)
- Conformance checking implementations (Token Replay, Alignment, Footprints)
- All I/O format handlers (XES, CSV, JSON, Parquet, OCEL)
- Statistical analysis and performance metrics
- Comprehensive test suite (274 tests, 95.6% pass rate)
- Property-based testing framework with quickcheck
- Benchmarking suite and performance evaluation
- Documentation (API docs, guides, architecture specifications)
- Python bindings via PyO3

**Evaluation & Validation:**
- Benchmark design and execution on standard datasets (BPIC 2012/2018, UCI)
- Comparative analysis vs Python pm4py
- Correctness verification (±<1e-11 relative error)
- Memory efficiency analysis (86-89% reduction)
- Scaling tests (100K to 100M events)
- Formal correctness proofs via property testing

**Academic Materials:**
- Research paper (12 pages, 30+ citations)
- Evaluation metrics documentation
- Reproducibility guide and supplementary materials
- Cover letter and submission materials
- Presentation materials

---

## Collaborators & Acknowledgments

### Direct Collaborators

None directly on this work, though the author acknowledges inspiration from:

- **Roberto** (MIOSA): Feedback on architectural design choices
- **Straughter** (MIOSA team): Input on enterprise deployment scenarios

### Intellectual Foundations

This work builds on:

- **van der Aalst, W. M.** (RWTH Aachen): Process mining theory and foundational algorithms
- **Leemans, S. J. J.** (Eindhoven University): pm4py library design and Python API
- **Matsakis, N. D.** (Rust): Type system design enabling formal verification
- **Bezanson, J.** et al.: Insights from Julia language for scientific computing in systems languages

### Dataset Contributors

- **4TU Data Center:** BPIC datasets (2012, 2018)
- **UCI Machine Learning Repository:** Road traffic prediction dataset

---

## Competing Interests

**Declaration of Interests:**

1. **No financial interests** in competing process mining tools
2. **No employment conflicts** with any commercial PM vendors
3. **AGPL-3.0 licensing** ensures GPL-compatible use; dual licensing available upon request
4. **No patents filed or pending** on core algorithms (all are academic/published)
5. **Crates.io publication** is free and open-source compatible

**Funding & Support:**
- This work was conducted independently without external funding
- No grants or sponsorships influenced the research direction
- Used open-source tools (Rust, crates.io ecosystem, GitHub)

---

## Availability Statement

**Data Availability:**
- Source code: https://github.com/seanchatmangpt/pm4py-rust
- Benchmark datasets: Public (BPIC 2012/2018 via 4TU, UCI via archive.ics.uci.edu)
- Reproducibility: Complete (REPRODUCIBILITY_GUIDE.md)

**Code Availability:**
- Published on crates.io: https://crates.io/crates/pm4py
- License: AGPL-3.0-or-later (dual licensing available)
- Development: Active with regular maintenance

**Correspondence:**
For questions regarding the research or reproduction:
- Email: info@chatmangpt.com
- GitHub Issues: https://github.com/seanchatmangpt/pm4py-rust/issues

---

## Publication Ethics

This manuscript:
- Has NOT been previously published
- Is NOT simultaneously under review elsewhere
- Represents original work by the author(s)
- Includes honest discussion of limitations and future work
- Provides complete reproducibility materials
- Follows COPE (Committee on Publication Ethics) guidelines

---

## Suggested Reviewers

We suggest the following experts as potential reviewers:

### Process Mining Domain
1. **Sander Leemans** (Eindhoven University)
   - Expertise: PM4Py architecture, inductive mining
   - Email: s.j.j.leemans@tue.nl

2. **Boudewijn van Dongen** (Eindhoven University)
   - Expertise: Conformance checking, alignment algorithms
   - Email: b.f.v.dongen@tue.nl

3. **Wil van der Aalst** (RWTH Aachen)
   - Expertise: Process mining fundamentals, Petri nets
   - Email: wvdaalst@pads.rwth-aachen.de

### Systems & Performance Domain
1. **Graydon Hoare** (Rust Language)
   - Expertise: Rust language design, memory safety
   - Affiliation: Mozilla/Rust Foundation

2. **Niko Matsakis** (Rust Type System)
   - Expertise: Rust compiler, type system correctness
   - Email: nmatsakis@mozilla.com

### High-Performance Computing Domain
1. **Julia Stoyanovich** (University of Pennsylvania)
   - Expertise: High-performance data systems, benchmarking
   - Email: jstoyan@upenn.edu

2. **Tim Kraska** (MIT)
   - Expertise: Database systems, performance evaluation
   - Email: kraska@mit.edu

---

## Author Commitments

**Post-Publication Commitments:**

1. **Maintenance:** Ongoing updates and bug fixes for 3+ years post-publication
2. **Support:** Response to reviewer/reader questions within 2 weeks
3. **Data Sharing:** All artifacts remain publicly available
4. **Code Quality:** Maintain >90% test coverage and clippy clean status
5. **Documentation:** Keep API documentation synchronized with implementation

**Presentation:**
- Available to present at conferences (virtual or in-person)
- Willing to participate in workshop discussions
- Open to conducting tool demonstrations

---

## Document Metadata

- **Last Updated:** 2026-03-24
- **Status:** FINAL
- **Version:** 1.0
- **Prepared for:** Academic peer review and conference submission

---

**Signature:**

Sean Chatman

Date: March 24, 2026
