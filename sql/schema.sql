-- BDAI (Bangla Dataset & AI Platform) Schema for Supabase
-- Tables: users, team_members, news_articles, vacancies, research_objectives, activity_logs

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Users Table (Admin, Moderator, Member)
CREATE TABLE IF NOT EXISTS public.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('Admin', 'Moderator', 'Member')),
    avatar TEXT DEFAULT '/team/miskat.jpg',
    department TEXT NOT NULL DEFAULT 'Department of CSE, University of Chittagong',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2. Team Members Table (Personnel with fully editable designations)
CREATE TABLE IF NOT EXISTS public.team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    designation TEXT NOT NULL, -- Fully editable free text (e.g. SPM, ASPM, Associate Professor, PhD Fellow)
    role TEXT,
    category TEXT,
    institution TEXT NOT NULL DEFAULT 'Department of Computer Science and Engineering, University of Chittagong',
    email TEXT,
    bio TEXT,
    image TEXT DEFAULT '/team/miskat.jpg',
    scholar_url TEXT,
    linkedin_url TEXT,
    display_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. News & Milestones Articles
CREATE TABLE IF NOT EXISTS public.news_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    excerpt TEXT NOT NULL,
    content TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('news', 'event', 'workshop', 'announcement')),
    publish_date DATE NOT NULL DEFAULT CURRENT_DATE,
    author TEXT NOT NULL DEFAULT 'BIKE Lab',
    status TEXT NOT NULL DEFAULT 'published' CHECK (status IN ('published', 'draft')),
    featured BOOLEAN NOT NULL DEFAULT false,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 4. Vacancies & e-Tenders / Notices (Free-form notice_type)
CREATE TABLE IF NOT EXISTS public.vacancies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    department TEXT NOT NULL DEFAULT 'Department of Computer Science and Engineering',
    work_package TEXT NOT NULL DEFAULT 'HEAT-13211-CU ATF Sub-Project',
    notice_type TEXT NOT NULL, -- Free text notice type (e.g. e-Tender Notice (OTM Goods), Research Fellowship)
    location TEXT NOT NULL DEFAULT 'University of Chittagong, Chattogram',
    deadline DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    description TEXT NOT NULL,
    requirements TEXT[] NOT NULL DEFAULT '{}',
    applicant_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. Research Objectives & Deliverables
CREATE TABLE IF NOT EXISTS public.research_objectives (
    id TEXT PRIMARY KEY, -- e.g. "OB1", "OB2"
    title TEXT NOT NULL,
    details TEXT NOT NULL,
    researcher TEXT NOT NULL,
    sector TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'in-progress' CHECK (status IN ('in-progress', 'completed', 'planned')),
    progress INT NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    deliverables INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 6. Activity Logs
CREATE TABLE IF NOT EXISTS public.activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action TEXT NOT NULL,
    entity TEXT NOT NULL,
    target_name TEXT NOT NULL,
    user_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON public.users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON public.users(role);
CREATE INDEX IF NOT EXISTS idx_team_designation ON public.team_members(designation);
CREATE INDEX IF NOT EXISTS idx_news_category ON public.news_articles(category);
CREATE INDEX IF NOT EXISTS idx_news_slug ON public.news_articles(slug);
CREATE INDEX IF NOT EXISTS idx_vacancies_status ON public.vacancies(status);
CREATE INDEX IF NOT EXISTS idx_vacancies_notice_type ON public.vacancies(notice_type);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION public.handle_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_users_updated_at ON public.users;
CREATE TRIGGER set_users_updated_at
BEFORE UPDATE ON public.users
FOR EACH ROW EXECUTE FUNCTION public.handle_updated_at();

DROP TRIGGER IF EXISTS set_team_updated_at ON public.team_members;
CREATE TRIGGER set_team_updated_at
BEFORE UPDATE ON public.team_members
FOR EACH ROW EXECUTE FUNCTION public.handle_updated_at();

DROP TRIGGER IF EXISTS set_news_updated_at ON public.news_articles;
CREATE TRIGGER set_news_updated_at
BEFORE UPDATE ON public.news_articles
FOR EACH ROW EXECUTE FUNCTION public.handle_updated_at();

DROP TRIGGER IF EXISTS set_vacancies_updated_at ON public.vacancies;
CREATE TRIGGER set_vacancies_updated_at
BEFORE UPDATE ON public.vacancies
FOR EACH ROW EXECUTE FUNCTION public.handle_updated_at();

DROP TRIGGER IF EXISTS set_objectives_updated_at ON public.research_objectives;
CREATE TRIGGER set_objectives_updated_at
BEFORE UPDATE ON public.research_objectives
FOR EACH ROW EXECUTE FUNCTION public.handle_updated_at();
