-- Seed initial BDAI data matching the live application

-- 1. Insert Initial Users (Admin, Moderator)
-- Password for all default accounts is: admin123
-- Stored as bcrypt hash: $2a$12$1jR4mYp7c9kX8tW2qFzCReE6cWjT1vM0uO3bL7pQ8eN2sA4xZ9uCy
INSERT INTO public.users (id, name, email, password_hash, role, avatar, department, status)
VALUES
    ('a0000000-0000-0000-0000-000000000001', 'Prof. Dr. Rudra Pratap Deb Nath', 'rudra@cu.ac.bd', '$2a$12$1jR4mYp7c9kX8tW2qFzCReE6cWjT1vM0uO3bL7pQ8eN2sA4xZ9uCy', 'Admin', '/team/rudra.jpg', 'Sub-Project Manager (SPM), Department of CSE, CU', 'active'),
    ('a0000000-0000-0000-0000-000000000002', 'Dr. Abu Nowshed Chy', 'nowshed@cu.ac.bd', '$2a$12$1jR4mYp7c9kX8tW2qFzCReE6cWjT1vM0uO3bL7pQ8eN2sA4xZ9uCy', 'Admin', '/team/nowshed.png', 'Alternate SPM (ASPM), Department of CSE, CU', 'active'),
    ('a0000000-0000-0000-0000-000000000003', 'Miskat Hasan', 'miskat.cse@cu.ac.bd', '$2a$12$1jR4mYp7c9kX8tW2qFzCReE6cWjT1vM0uO3bL7pQ8eN2sA4xZ9uCy', 'Moderator', '/team/miskat.jpg', 'Research Assistant & Full-stack Engineer, BIKE Lab', 'active'),
    ('a0000000-0000-0000-0000-000000000004', 'Sayed Hossain', 'sayed.fellow@cu.ac.bd', '$2a$12$1jR4mYp7c9kX8tW2qFzCReE6cWjT1vM0uO3bL7pQ8eN2sA4xZ9uCy', 'Moderator', '/team/sayed.jpg', 'PhD Research Fellow (KG-RAG Domain)', 'active')
ON CONFLICT (email) DO NOTHING;

-- 2. Insert Initial Team Members (Personnel with custom editable designations)
INSERT INTO public.team_members (id, name, designation, role, category, institution, email, bio, image, scholar_url, display_order)
VALUES
    ('b0000000-0000-0000-0000-000000000001', 'Prof. Dr. Rudra Pratap Deb Nath', 'SPM & Professor', 'Sub-Project Manager (SPM)', 'lead', 'Department of Computer Science and Engineering, University of Chittagong', 'rudra@cu.ac.bd', 'Professor at the Department of Computer Science & Engineering, CU and SPM for the BDAI project.', '/team/rudra.jpg', 'https://scholar.google.com/citations?user=rudra', 1),
    ('b0000000-0000-0000-0000-000000000002', 'Dr. Abu Nowshed Chy', 'ASPM & Associate Professor', 'Alternate Sub-Project Manager (ASPM)', 'co-lead', 'Department of Computer Science and Engineering, University of Chittagong', 'nowshed@cu.ac.bd', 'Associate Professor at CSE, University of Chittagong and ASPM overseeing AI research architectures.', '/team/nowshed.png', 'https://scholar.google.com/citations?user=nowshed', 2),
    ('b0000000-0000-0000-0000-000000000003', 'Dr. Mohammad Sanaullah Chowdhury', 'Professor & Senior Researcher', 'Senior Researcher', 'researcher', 'Department of Computer Science and Engineering, University of Chittagong', 'sanaullah@cu.ac.bd', 'Professor at CSE, University of Chittagong. Expert in applied algorithms and data systems.', '/team/sanaullah.jpg', 'https://scholar.google.com/citations?user=sanaullah', 3),
    ('b0000000-0000-0000-0000-000000000004', 'Miskat Hasan', 'Research Assistant', 'Research Assistant & Platform Lead', 'research-assistant', 'BIKE Lab, Department of CSE, University of Chittagong', 'miskat.cse@cu.ac.bd', 'Research Assistant at BIKE Lab working on LLM benchmarks, Knowledge Graphs, and frontend portals.', '/team/miskat.jpg', 'https://scholar.google.com/citations?user=miskat', 4),
    ('b0000000-0000-0000-0000-000000000005', 'Sayed Hossain', 'PhD Fellow', 'Doctoral Researcher', 'research-assistant', 'Department of Computer Science and Engineering, University of Chittagong', 'sayed.fellow@cu.ac.bd', 'PhD Fellow focusing on Cross-Sectoral Knowledge Graph architectures and RAG hallucinations.', '/team/sayed.jpg', 'https://scholar.google.com/citations?user=sayed', 5),
    ('b0000000-0000-0000-0000-000000000006', 'Sadia Afrin', 'Data Annotator & Analyst', 'Research Assistant', 'research-assistant', 'Department of Computer Science and Engineering, University of Chittagong', 'sadia.annotator@cu.ac.bd', 'Specializes in high-quality Bengali linguistic corpus curation, entity relation labeling, and validation.', '/team/sadia.jpg', 'https://scholar.google.com/citations?user=sadia', 6)
ON CONFLICT (id) DO NOTHING;

-- 3. Insert News & Milestones
INSERT INTO public.news_articles (id, title, slug, excerpt, content, category, publish_date, author, status, featured, tags)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'e-Tender Notice for Procurement of AI Workstations (HEAT-13211-CU)', 'etender-notice-ai-workstations-2026', 'The BDAI project has published an e-Tender notice for supply and installation of enterprise AI workstations.', 'The project has published an official e-Tender notice for the supply and installation of enterprise GPU AI workstations under the HEAT-13211-CU ATF sub-project.\n\nTender proposals must be submitted through the national e-GP system portal (www.eprocure.gov.bd). All qualified bidders are encouraged to participate before the submission closing date.', 'announcement', '2026-06-02', 'BDAI Procurement Cell', 'published', true, ARRAY['Procurement', 'e-Tender', 'Workstations', 'Hardware']),
    ('c0000000-0000-0000-0000-000000000002', 'Workshop on Bangla Knowledge Graphs & LLM Hallucination Mitigation', 'workshop-bangla-kg-llm-2026', 'A hands-on workshop on constructing domain-specific Knowledge Graphs for Bangla NLP and LLM RAG integration.', 'Researchers and fellows from BIKE Lab participated in an intensive 2-day technical workshop detailing ontology creation, RDF serialization, and Neo4j graph pipeline construction for Bengali public health and legal domains.', 'workshop', '2026-04-12', 'BIKE Research Team', 'published', false, ARRAY['Knowledge Graphs', 'Workshop', 'Bangla NLP', 'Neo4j'])
ON CONFLICT (slug) DO NOTHING;

-- 4. Insert Vacancies / Notices
INSERT INTO public.vacancies (id, title, department, work_package, notice_type, location, deadline, status, description, requirements, applicant_count)
VALUES
    ('d0000000-0000-0000-0000-000000000001', 'Supply and installation of AI workstations (e-Tender Notice OTM Goods)', 'Department of Computer Science and Engineering', 'HEAT-13211-CU ATF Sub-Project', 'e-Tender Notice (OTM Goods)', 'University of Chittagong, Chattogram', '2026-06-17', 'open', 'Procurement of high-performance GPU AI workstations under memo HEAT/CU/PIN13211/G-03.11/2025-2026 via National e-GP System Portal.', ARRAY['Submission strictly via National e-GP System Portal (www.eprocure.gov.bd)', 'Valid Trade License and Tax Identification Number (TIN)', 'Authorized OEM distributor partner certification for enterprise GPU hardware'], 5),
    ('d0000000-0000-0000-0000-000000000002', 'PhD and MPhil Research Fellowships under the BDAI project (HEAT)', 'BIKE Lab & CSE Department', 'WP3 & WP5: Cross-Sectoral KGs & KG-RAG', 'Research Fellowship', 'University of Chittagong', '2026-08-30', 'open', 'Funded doctoral and masters positions for candidates pursuing state-of-the-art research in Knowledge Graph construction and LLM hallucination mitigation.', ARRAY['M.Sc. or B.Sc. in Computer Science & Engineering with excellent academic standing', 'Strong programming proficiency in Python, PyTorch, and Neo4j', 'Demonstrated enthusiasm for NLP and graph neural networks'], 18)
ON CONFLICT (id) DO NOTHING;

-- 5. Insert Research Objectives
INSERT INTO public.research_objectives (id, title, details, researcher, sector, status, progress, deliverables)
VALUES
    ('OB1', 'Open Data Quality Measurement Framework', 'An Open data quality measurement framework for assessing completeness and freshness of national open datasets.', '(Masters-1)', 'Data Governance', 'completed', 100, 3),
    ('OB2', 'SMART Data Ecosystem for Bangladesh', 'A full-fledged SMART data ecosystem tailored to Bangladesh data.', '(PhD-1)', 'National Infrastructure', 'in-progress', 65, 4),
    ('OB3', 'Ontology for Cross-Domain Data Integration', 'An ontology that unifies multi-domain knowledge from agriculture, healthcare, and education.', '(Masters-2)', 'Semantic Web', 'in-progress', 45, 2),
    ('OB4', 'Cross-Sectoral Knowledge Graphs & RAG', 'Development of cross-sectoral Knowledge Graphs for Bengali Question-Answering and KG-RAG.', '(PhD-2)', 'AI & NLP', 'in-progress', 30, 5)
ON CONFLICT (id) DO UPDATE SET progress = EXCLUDED.progress, status = EXCLUDED.status;
