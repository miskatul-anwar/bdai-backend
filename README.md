# BDAI Platform — Rust Backend (`bdai_backend`)

High-performance, async Rust backend for the **BDAI (Bangla Dataset & AI Platform)** built with **Axum**, **Tokio**, and **SQLx**, backed by **Supabase PostgreSQL**.

---

## 🚀 Features

- **Blazing Fast Axum Framework**: Async HTTP services with Tokio, Tower middleware, and CORS.
- **Supabase PostgreSQL Integration**: Direct connection pooling with SQLx, auto-migrations, and seed data.
- **Strict Role-Based Access Control (RBAC)**:
  - **Only Admin can Add or Remove** an Admin, Moderator, Member, and all kinds of employees.
  - **Moderators** can create and update news, tenders/notices, and research milestones.
  - **Members** have read-only access.
- **Google OAuth 2.0 Whitelist Authentication**:
  - Sign in with institutional (`@cu.ac.bd`) or personal Google accounts.
  - **Strict Admin Whitelist Enforced**: Only Google accounts whose emails have previously been added by an Admin are granted access. Any unregistered account is rejected with HTTP `403 Forbidden`.
  - Automatic avatar synchronization from Google profiles.
- **Dynamic Employee Designations**: No fixed category enums. Any designation (*SPM, ASPM, Associate Professor, PhD Fellow, Research Assistant, Data Annotator*, etc.) is supported.
- **Free-Form Notice Types**: Vacancy and e-Tender notices support arbitrary text notice types (*e-Tender Notice (OTM Goods)*, *Research Fellowship*, *Job Circular*, *RFQ*).
- **Secure JWT Authentication**: Bcrypt password hashing and stateless token issuance.

---

## 📦 Project Structure

```
bdai_backend/
├── Cargo.toml
├── .env.example
├── .env
├── README.md
└── src/
    ├── main.rs            # Server bootstrap, CORS, graceful shutdown
    ├── config.rs          # Environment configuration
    ├── db.rs              # SQLx connection pool & auto-migration
    ├── error.rs           # Standardized AppError & JSON HTTP responses
    ├── auth.rs            # JWT encoding/decoding & bcrypt verification
    ├── middleware.rs      # RBAC extractors: CurrentUser, AdminOnly, EditorUser
    ├── state.rs           # Shared Axum AppState
    ├── models/            # Domain entities (User, Team, News, Vacancy, etc.)
    └── routes/            # REST API endpoints
        ├── auth_routes.rs
        ├── user_routes.rs
        ├── team_routes.rs
        ├── news_routes.rs
        ├── vacancy_routes.rs
        ├── objective_routes.rs
        ├── activity_routes.rs
        └── health_routes.rs
```

---

## 🛠️ Quickstart

### 1. Database Setup (Supabase)

#### Option A: Local Supabase with CLI (Recommended for Development)
```bash
# In the root bdai directory:
supabase start
```
This starts local Supabase containers (PostgreSQL on port `54322`, Studio on `http://localhost:54323`).

To apply the schema and seed data manually if needed:
```bash
supabase db reset
```

#### Option B: Supabase Cloud Project
In your Supabase project dashboard (Settings ➔ Database ➔ Connection String ➔ URI / Transaction Pooler), copy your database URL and paste it in `bdai_backend/.env`:
```env
DATABASE_URL=postgres://postgres.[PROJECT_REF]:[PASSWORD]@aws-0-ap-southeast-1.pooler.supabase.com:6543/postgres?sslmode=require
```

### 2. Configure Environment

Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```
Ensure `DATABASE_URL` matches your local or Supabase cloud instance.

### 3. Run the Backend

```bash
cd bdai_backend
cargo run
```
The server will start listening on **`http://localhost:8080`**.

---

## 🔒 Security & RBAC Matrix

| Endpoint | Method | Required Role | Description |
| :--- | :--- | :--- | :--- |
| `/api/auth/login` | `POST` | Public | Login with email and password |
| `/api/auth/google` | `POST` | Public (Whitelist Enforced) | Sign in with Google (rejects unregistered emails) |
| `/api/auth/google/url` | `GET` | Public | Get Google OAuth 2.0 consent URL |
| `/api/auth/google/callback` | `GET` | Public | Handle Google OAuth redirect callback |
| `/api/auth/me` | `GET` | Authenticated | Get current authenticated user profile |
| `/api/users` | `GET` | Authenticated | List all user accounts |
| `/api/users` | `POST` | **Admin Only** | Create an Admin, Moderator, or Member |
| `/api/users/:id` | `PUT` | **Admin Only** | Edit user profile or reassign role |
| `/api/users/:id` | `DELETE` | **Admin Only** | Remove user account (cannot delete self) |
| `/api/team` | `GET` | Public | List team members with editable designations |
| `/api/team` | `POST` | **Admin Only** | Add employee across all designations |
| `/api/team/:id` | `PUT` | Admin / Moderator | Update employee details |
| `/api/team/:id` | `DELETE` | **Admin Only** | Remove employee from roster |
| `/api/news` | `GET` | Public | List news and milestones |
| `/api/news` | `POST` | Admin / Moderator | Publish news article or announcement |
| `/api/news/:id` | `PUT` | Admin / Moderator | Update news article |
| `/api/news/:id` | `DELETE` | **Admin Only** | Remove news article |
| `/api/vacancies` | `GET` | Public | List vacancies and e-Tender notices |
| `/api/vacancies` | `POST` | Admin / Moderator | Post tender notice or fellowship vacancy |
| `/api/vacancies/:id` | `PUT` | Admin / Moderator | Update tender notice |
| `/api/vacancies/:id` | `DELETE` | **Admin Only** | Remove tender notice |
| `/api/objectives` | `GET` | Public | List research milestones & deliverables |
| `/api/objectives/:id` | `PUT` | Admin / Moderator | Update research milestone progress |
| `/api/activities` | `GET` | Authenticated | View system audit logs |
| `/api/health` | `GET` | Public | Service health & database connectivity check |

---

## 🔑 Default Accounts (Seed Data)

| Name | Email | Default Password | Role |
| :--- | :--- | :--- | :--- |
| Prof. Dr. Rudra Pratap Deb Nath | `rudra@cu.ac.bd` | `admin123` | **Admin** |
| Dr. Abu Nowshed Chy | `nowshed@cu.ac.bd` | `admin123` | **Admin** |
| Miskat Hasan | `miskat.cse@cu.ac.bd` | `admin123` | **Moderator** |
| Sayed Hossain | `sayed.fellow@cu.ac.bd` | `admin123` | **Member** |

---

## ☁️ Deploying to Render

The backend is packaged with an optimized multi-stage Docker build and a Render Blueprint specification (`render.yaml`).

### Prerequisites
1. A [Supabase](https://supabase.com) PostgreSQL database (Cloud or self-hosted).
2. A [Cloudinary](https://cloudinary.com) account for media storage.
3. A [Render](https://render.com) account.

---

### Method A: 1-Click Blueprint Deploy (Recommended)

1. Push your repository to GitHub / GitLab.
2. In your Render Dashboard, click **New +** ➔ **Blueprint**.
3. Select your repository. Render will automatically detect [`render.yaml`](file:///home/miskat/Projects/bdai/render.yaml).
4. Fill in the prompted secret values:
   - `DATABASE_URL`: Your Supabase PostgreSQL connection string (see below).
   - `CLOUDINARY_CLOUD_NAME`: Your Cloudinary cloud name.
   - `CLOUDINARY_API_KEY`: Your Cloudinary API key.
   - `CLOUDINARY_API_SECRET`: Your Cloudinary API secret.
5. Click **Apply**. Render will automatically build the multi-stage Docker container and launch the service with zero downtime.

---

### Method B: Manual Web Service Creation

If you prefer creating the service manually:
1. In the Render Dashboard, click **New +** ➔ **Web Service**.
2. Connect your Git repository.
3. Configure the following settings:
   - **Language / Runtime**: `Docker`
   - **Root Directory**: `bdai_backend` (or leave blank if repository root is the backend)
   - **Dockerfile Path**: `./Dockerfile`
   - **Docker Context**: `.`
   - **Instance Type**: `Free` or `Starter`
   - **Health Check Path**: `/api/health`
4. Add the following **Environment Variables**:

| Variable | Recommended / Required Value | Description |
| :--- | :--- | :--- |
| `PORT` | `10000` | Render dynamically forwards traffic to this port |
| `DATABASE_URL` | `postgres://postgres.[REF]:[PASS]@[HOST]:6543/postgres?sslmode=require` | Supabase Postgres Pooler URL |
| `JWT_SECRET` | *(Random 32+ character string)* | Secret for signing JWT authentication tokens |
| `JWT_EXPIRATION_HOURS` | `72` | Token validity duration |
| `CLOUDINARY_CLOUD_NAME` | *(Your cloud name)* | Cloudinary account name |
| `CLOUDINARY_API_KEY` | *(Your API key)* | Cloudinary API Key |
| `CLOUDINARY_API_SECRET` | *(Your API secret)* | Cloudinary API Secret |
| `CLOUDINARY_FOLDER` | `bdai` | Destination folder on Cloudinary |
| `CORS_ORIGINS` | `https://bdai.bike-csecu.com,http://localhost:3000,http://localhost:3001` | Allowed browser origins |

5. Click **Deploy Web Service**.

---

### 🌐 Supabase Connection String for Render

For cloud hosting on Render, Supabase requires TLS (`sslmode=require`).
Always use the **Transaction Pooler** URL (Port `6543`) or **Session Pooler** (Port `5432`) from your Supabase Dashboard:

> **Supabase Dashboard** ➔ **Project Settings** ➔ **Database** ➔ **Connection Pooling**:
> ```
> postgres://postgres.[PROJECT_REF]:[YOUR_PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres?sslmode=require
> ```

*Note: The backend automatically checks and executes schema migrations and seeds initial admin accounts if the tables are not yet present in your Supabase database.*

