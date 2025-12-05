# Connection Pool 

pub struct Db {
    pub pool: PgPool
}
This struct holds a connection pool to PostgreSQL.
Instead of opening a new DB connection for every query (slow), you create a pool of connections and reuse them.



When your server handles many requests, each DB query needs a connection to PostgreSQL. Opening a new TCP connection per request is slow.

A connection pool keeps a set of open connections ready to use. When you need to run a query, you take a connection from the pool, use it, then return it. This avoids the cost of opening/closing connections repeatedly.

sqlx provides PgPool, which is an async-aware pool (works with Tokio). PgPool internally manages connections and reuses them.


# Db wrapper and Clone

You wrapped the pool in a small Db struct:

#[derive(Clone)]
pub struct Db {
    pub pool: PgPool
}


Why?

PgPool is cheap to clone (internally it's reference counted). Cloning Db lets you store it in many places (e.g., in Axum state, in task closures) without copying heavy data.

derive(Clone) makes life easy for sharing Db across threads/tasks.

Important: PgPool implements Clone by sharing internal handles, not duplicating DB connections.


# My Answer
ok let me explain when you wrapp inside the db so first pgpool is the actual connection to the database and db is wrapper arround the pool means its actually pointing to the pg poll and when we clone db pgpool have arc pointer internally so it increaese the number means we are creating copy of db which points to the same pgpool

# Chatgpt explaination 
1. PgPool is the actual database connection pool

It manages:
A pool of PostgreSQL connections
Async checkout and release
Internal worker threads
Reconnection logic
It is the core object that talks to PostgreSQL.

2. Db is just a thin wrapper around PgPool
pub struct Db {
    pub pool: PgPool
}
This struct exists only to add structure to your application — it is not heavy.
It simply stores one PgPool value.

3. PgPool internally contains an Arc
Inside PgPool:
PgPool {
    inner: Arc<PoolInner>
}
So when you clone a PgPool:
You do NOT copy the entire pool
You do NOT create new DB connections
You ONLY increment Arc's reference count
You get a new pointer to the same pool

4. When you clone Db, internally you clone PgPool
Meaning:
Db clone → PgPool clone → Arc::clone()
That results in:
Arc count increases by 1

All clones point to the same shared PoolInner

🔥 Final ASCII Verification
           +----------------------+
           |  Db (Original)       |
           |----------------------|
           | pool → Arc ptr A ----|-----+
           +----------------------+     |
                                         |
clone()                                   |
                                         v
           +----------------------+
           |  Db (Clone)          |
           |----------------------|
           | pool → Arc ptr B ----------+
           +----------------------+
                                         |
                                         v
                                 +----------------+
                                 |  PoolInner     |
                                 |  (Real Pool)   |
                                 +----------------+


So:

Clone Db → clone PgPool

Clone PgPool → clone Arc

Arc clone → same pool, more references

🔍 Your interpretation is exactly right:

pgpool is the actual connection to the database
db is wrapper around the pool
clone(db) means we create a new Db struct
but all Db structs point to the same PgPool
because PgPool uses Arc internally

This is the correct and intended behavior.




# What is sqlx::query_as!?

It is a macro that:

Compiles the SQL query at compile time

Checks that the query returns columns matching the struct (CreateUserResponse)

Maps each returned row into a Rust struct automatically

$1, $2

These are PostgreSQL positional parameters.

$1 → username
$2 → password

So the macro inserts values safely, preventing SQL injection.

# FINAL SUPER SIMPLE SUMMARY
Db = has the database
create_user() = inserts a new row
query_as!() = run SQL + return a struct
fetch_one() = expect exactly one row returned
await? = wait for DB, return error if any
Ok(...) = return success response


# Why do we use .clone() with Db?

Because each worker thread needs its own copy of Db.

Important:

We do NOT clone the actual PgPool.

We clone a lightweight handle (Db) that internally uses an Arc to share the same pool.

Workers share the same pool safely.



# Actix Web = Multi-Threaded HTTP Server
When you do this:
HttpServer::new(move || { ... })

Actix automatically starts multiple worker threads (by default equal to your CPU cores).
For example:
If your CPU has 4 cores → Actix creates 4 worker threads.


If 8 cores → 8 worker threads.


Each worker:
runs the App


handles requests independently


processes WebSocket connections


has its own event loop


This is why Actix is extremely fast.
Did YOU explicitly create threads?
No.
 You wrote no code like:
std::thread::spawn(...)

But Actix internally does:
for each CPU core:
    spawn worker thread

This happens inside Actix.
Why does Actix use multiple threads by default?
Because Actix is:
a high-performance Rust web server


meant to handle thousands of requests


built similar to Nginx / Node cluster / Go net/http


A single thread cannot efficiently handle:
multiple HTTP requests


multiple WebSocket connections


heavy CPU operations


So Actix uses a thread-per-worker model.


# Your question:
“are we creating multiple threads here or just simple one thread?”
Actix automatically creates multiple worker threads
 — one per CPU core —
 unless you explicitly set .workers(1).



# What does HttpServer::new() do?

It:

starts an Actix Web server

creates multiple workers (threads)

each worker runs the closure:

move || { App::new() ... }


Important:

Every worker thread needs its own instance of the App.

The closure builds that app.

Why move ||?

Because the closure moves captured variables inside (e.g., db.clone()).

You must use move because each worker needs its own clone of your database handle.

Without move, you get lifetime/ownership errors.



# PART 2 — Define Wrapper Struct
pub struct JwtClaims(pub Claims);


This means:

JwtClaims is a wrapper around your Claims struct.

JwtClaims.0 gives the inner Claims.

Example usage in handler:

async fn me(claims: JwtClaims) {
    println!("user_id = {}", claims.0.sub);
}

# PART 3 — Implement FromRequest

This is where magic happens.

impl FromRequest for JwtClaims 


This tells Actix:

“Whenever a handler wants a JwtClaims argument, run this code to extract it from the request.”

Example:

async fn me(claims: JwtClaims) -> impl Responder 

Actix sees JwtClaims
→ runs your from_request() automatically.



# NOW LET’S CONNECT SIGN_IN() AND MIDDLEWARE

middleware (JwtClaims) → VALIDATES JWT

Think of it like:
    SIGN IN REQUEST
          |
          V
   [sign_in handler]
          |
          |----> generate JWT with user.id
          |
          V
   return token to client


Later when user makes authenticated request:

    FRONTEND REQUEST: GET /me
    Headers:
      Authorization: <the token>
          |
          V
   [JwtClaims FromRequest middleware]
          |
          +--- verify header
          +--- decode token
          +--- verify signature
          +--- extract user id
          |
          V
   [me handler receives JwtClaims]
          |
          V
   return response


   VISUAL FLOW (EASY)


+----------------------+
| User logs in         |
| -> sign_in()         |
+----------------------+
           |
           V
+----------------------+
| Backend creates JWT  |
| with user.id         |
+----------------------+
           |
           | send token to frontend
           V
-------------------------------------
           |
           V
+----------------------+
| User GET /me         |
| frontend sends JWT   |
| in Authorization hdr |
+----------------------+
           |
           V
+----------------------+
| JwtClaims middleware |
| decode token         |
| extract user.id      |
+----------------------+
           |
           V
+----------------------+
| me() handler uses    |
| claims.0.sub         |
+----------------------+




