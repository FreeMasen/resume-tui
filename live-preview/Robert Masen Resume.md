# Robert Masen

|Email|Github|LinkedIn|
|:----|:----:|-------:|
|<r.f.masen@gmail.com>|[FreeMasen](https://github.com//FreeMasen "github")|[robertmasen](https://linkedin.com/in//robertmasen "linkedin")|

## Work

### Vultr

July 2025 - Current

**Staff Software Engineer**

* **Virtual Machine Metadata Server**
  *Ported a Virtual Machine Metadata server from PHP to Rust*

- **Spearhead Rust project workspace organization**
  *Lead an effort to transition a proof of concept rust hypervisor project to a full cargo workspace with
  multiple binary projects and large amounts of shared code*

* **Ceph guest metrics from host kernel**
  *Developed a kernel patch to provide network disk performance metrics via the sys-fs debug interface
  for accessing performance of a virtual machine guest from the host*

- **Architect Real Time Inventory System**
  *Designed and developed a system for linux virtual machine hosts to report guest resource
  allocation in real time*

* **Develop high performance capacity inventory and vm host selection service**
  *Using real-time data published by virtual machine hosts for evaluating fleet wide capacity
  and selecting the most appropriate host for that customer, including pinned CPU and GPU offerings.*

### SmartThings

March 2020 - July 2025

**Staff Software Engineer**

* **Hub Local Metrics**
  
  *Implementing a memory efficient, Rust metrics collector for an embedded linux target*
  
  To gain additional visibility into the performance of our embedded Linux devices
  deployed across the world, I developed a system for collecting and reporting metrics for our fleet.
  
  This system was constrained by the needs of the device's relatively low memory and long expected
  lifetime, meaning keeping a time series either on disk or in memory was unavailable. To achieve some
  level of visibility, I developed a solution that would allow queries to be sent to devices using
  Launch Darkly, once received the query would start collecting any metrics events in a low overhead
  report. This report would then be emitted to our platform's cloud for post-processing.
  
  For counters the report can calculate its state on the fly quite trivially. For gauges, I developed
  a solution that would calculate the min, max, and average values on the fly. For histogram events,
  the *HdrHistogram* crate provided the calculations with a fixed size memory footprint with the
  limitation that all values need to be provided as an integer. Since the `metrics` crate only
  provides histogram data as a 64-bit floating point number I developed a solution to for converting
  these float values to/from a u64 with an acceptable amount of data loss.

- **Lua Sandbox**
  
  *Develop and maintain a Lua sandbox in Rust for executing untrusted code on embedded linux devices
  using gRPC for interactions with the wider platform*
  
  To enable our platform to be as flexible as possible, I developed and performed
  maintenance on a sand-boxed execution environment for Lua scripts to interact with smart home
  devices. This project used seccomp, nsjail, runtime checks, and an RPC layer to isolate scripts
  from the rest of the product.
  
  In this system each script (aka "driver") is executed in its own child process with an anonymous UDS
  pair acting as the RPC connection. Protected operations include creating and/or communicating on
  TCP/UDP sockets sending, receiving messages from the Zigbee, Thread, and ZWave radios, and
  communicating device status to/from our cloud platform.

* **Edge HTTP API**
  
  *Built an HTTP API in Rust running on edge devices to act as a cloud service*
  
  Build an Actix-Web based HTTP server for local communication and processing of
  cloud originated messages on the edge. A service was deployed in the cloud to receive messages
  destined for a device on the edge, this service would look up the the device by ID and then
  forward the request through a persistent connection.
  
  The device's HTTP server would receive this request, perform authorization of both client and
  principal (typically a user) before executing the endpoint and returning a response through the same
  persistent connection. This system was developed to be flexible enough to act as an RPC system on a
  Tizen/Android based devices as well as the potential for end users to access the interface on the
  local area network. The authorization mechanism uses a sequence of attempts to authorize the
  principal including local evaluation of a subset of definitions, caching previous authorizations and
  a call out to the cloud service that acts as a platform single source of truth.

- **RPC Rewrite**
  
  *To address performance issues on embedded linux hardware, developed and transitioned the platform
  to use a purpose build RPC mechanism to replace Tonic and gRPC.*
  
  The *Lua Sandbox*'s RPC mechanism was initially developed using an off the shelf
  library for gRPC. This solution seemed like the right choice initially since gRPC provides a very
  flexible interface, however the constraints of an embedded linux environment and using a Unix Domain
  Socket(UDS) pair as the underlying transport made some parts of gRPC less than ideal. To replace
  this system, I developed one initially using a UDS pair with datagram semantics which when initially
  deployed resulted in an average 20% performance gain however we discovered that some of our
  messaging between parent and child process was much too large to fit into a reasonably sized
  datagram and in some cases was too large for the default OS maximum of 200+ kb. This lead to a
  second iteration that used a UDS pair with stream semantics to allow for larger messages. This
  stream based implementation used a 4 byte length as a prefix to all messages.

* **Utility Services**
  
  *Develop a significant number of Rust platform utility and services to address shared needs across
  multiple teams*
  
  As with any job, some of the work I have done doesn't quite fit into its own category, the following
  projects describe some of the smaller projects I developed that had a significant impact but were not
  quite large enough to warrant their own entry.
  
  #### **Flexible Key-Value Store**
  
  A single sqlite table was used with a composite primary key of two TEXT columns, the first
  being a namespace while the second a unique identifier in that namespace. A Third TEXT column
  contains the values encoded as JSON. A handle to the store would be constructed with the namespace
  key so that consumers could avoid concerns about key uniqueness.
  
  #### **IP Source**
  
  The application we developed could be deployed to a number of products, some of which acted as
  a Router in end-user's homes. This meant we needed a way to communicate our current IP address
  on the local area network in a uniform way avoiding potentially providing the address that
  would be accessible from outside of a user's home.
  
  This service would act as the single source of truth for what IP address the device should be
  using based on the an interface name configured by either a supervisor application or the
  operating system's read only disk partition.
  
  Consumers could request the current IP address as needed or subscribe to events to know when
  the current IP address had changed.
  
  #### **Asset Manager**
  
  Devices deployed in the field would need to request and maintain assets associated with
  different sub-systems. One example is the Lua script files that represent an Edge Driver
  (see Lua Sandbox) or the Matter firmware assets.
  
  This service exposes a HashMap-like interface for accessing the root of these assets on disk
  based on a unique identifier and a version identifier key pair. If an asset is requested with
  a pair of identifiers that are unknown, the asset is requested from its source and once on disk
  the requester is provided the new path. All assets are held on disk for a configurable length of
  time to avoid potential flash-wear and/or extra work for assets that may be reverted quickly to
  a previous version.
  
  #### **Stable UUIDs in Tests**
  
  I developed a set of functions that would generate v4 UUIDs by hashing some input text and then
  built a series of helper functions that would use use test thread names as the base value for this
  input text but could be extended. This enabled developers to no longer have to maintain manual UUID
  definitions and/or work around the randomness provided by UUIDs while maintaining a stable value
  across runs of the same tests.

- **Documentation and Design**
  *Using a mixture of `rust-doc`, Confluence, PlantUML, Mermaid.js, and Lucid Chart I
  documented existing behavior, communicated proposed changes, and design documents for future
  opportunities.*

* **Formal Mentor**
  
  *Mentored an Associate Software Engineer with the goal of getting promoted*
  
  As part of an official effort to improve Diversity, Equity and Inclusion at SmartThings
  I participated in a formal mentorship program. As part of this program, I was paired with a
  junior-level engineer whose goal was to get promoted in the next promotion cycle. To achieve this
  goal we worked through some of the ways that they could increase their visibility, track their own
  performance, and tell a compelling story about why they deserve to be promoted.

### Solution Design Group

February 2019 - March 2020

**Software Development Consultant**

* **Develop ASP.net Application**
  
  *Build new features into a ASP.net payment portal*
  
  Partner with client to build new features into an ASP.Net Web API, Angular and Web
  Forms applications. Participate in backlog sizing and retrospectives.

- **Modernization and Consistency**
  
  *Introduce static analysis utilities to increase code quality and consistency*
  
  Performed multi-version, breaking change upgrades of the Angular front-end framework
  including converting the shared HTTP interface from Observables to Promises.
  
  Introduced TSlint to CI to improve overall code quality and consistency.

* **Create Community**
  
  *Started a Programming Languages Community*
  
  As consultants, employees of SDG are not always co-located, to create a sense of
  community employees are encouraged to collect after hours to share interests. A good example of one
  of these groups was a jam session, where musically inclined folks can get together and play music
  together. In my time with the company, I started a community around Programming Languages where we
  would meet monthly to share and discuss different programming languages like Rust, SML, and Closure.

### United Subcontractors

August 2017 - December 2018

**Software Developer**

* **Develop ASP.net Web Forms**
  *Build new features and perform bug fixes on an existing ASP.NET Web Forms application*

- **React**
  
  *Transition existing application functionality into a React application using Typescript*
  
  Work with business analyst to develop a modernized version of a ASP.net WebForms system
  for job site inventory management. Building a new API backed using ASP.net for the backend
  and React (using TypeScript) for the front end.

## Open Source

* **Rusty ECMA**
  
  *JavaScript Parsing/Generating libraries*
  
  As an excuse to learn more about how parsers work in software, I started this project with the
  primary goal of providing a flexible, general purpose suite of libraries for interacting with JavaScript source code.
  
  The project consists of a handful of crates that I've tried to seperate logically to allow user to opt-in to the code
  they want to use.
  
  * **RESSA**
    
    *JavaScript Parser in Rust*
    
    This is probably the most obvious entrpypoint for end users in the project, it
    consists of a Parser that can produce an Abstract Syntax Tree of JavaScript code. It uses
    a recursive decent pattern to achieve the parsing and performs some level of code validation, for
    example it will error when encountering duplicate function argument identifiers.
  
  * **RES-Regex**
    
    *A parser of JavaScript Regular Expression Literals*
    
    This project validates the correctness of JavaScript flavored Regular Expression
    literals.
  
  * **RESAST**
    
    *JavaScript AST in Rust*
    
    This project represents the Abstract Syntax Tree of JavaScript code.
  
  * **RESW**
    
    *JavaScript Writer over RESAST data structures*
    
    This project is capabile of consuming the AST provided by RESAST and outputting
    JavaScript text, with a bit of configuability.
  
  * **RESS**
    
    *JavaScript Tokenizer in Rust*
    
    This is probably the most obvious entrypoint for contributors in the project it
    provides a `Scanner` that will produce a stream of `Token`s representing the discrete segments of
    the provided JavaScript. It even has the ability to detect if any `/` represents numeric division or
    is the beginning of a regular expression.

* **Cosock**
  
  *Lua couroutine I/O libraries*
  
  Cosock is both a project and an organization. The Project is a coroutine executor
  based on the [LuaSocket API](https://github.com/lunarmodules/luasocket) and uses a polling via
  `select` to schedule coroutines. It provides udp, tcp, and tls socket facilities and an MPSC channel
  implementation. The Cosock organization includes projects that build upon these primitives to
  provide HTTP facilities.
  
  * **Luxure**
    
    *HTTP Framework*
    
    This project was my initial entry into the Cosock organization, initially to support
    a SmartThings Edge Driver that acts as a virtual smart switch via a locally hosted HTTP server. This
    project provides and Express-like API for defining an HTTP server, where a `Server` is
    instantiated and provided an IP address, port pair to listen on. Endpoints are defined by calling
    methods on the `Server` corresponding to the HTTP Methods (GET, POST, PUT, etc). Finally the
    `Server` can be driven either via `run` which will occupy the whole process or can be driven
    manually via `tick` which will accept a single incoming request which allows the server to be
    used as a component of a process.
  
  * **Lustre**
    
    *Websockets in Lua*
    
    I started this library as an experiement in parsing a data format that was slightly
    more complicated than HTTP. Currently, there is only a client side implementation but supports all
    features using cosock primitives to allow for concurrency.
  
  * **Luncheon**
    
    *HTTP Parsing, Serialization and Types*
    
    I developed this project to act as a shared set of definitions to by used by an
    HTTP server implementation called Luxure and a WebSocket implementation called Lustre.
    
    This project provides an abstraction for `HttpMessage`s, which are inherited by the Request and
    Response types. These types also share a definition for a `Headers` map. All types can be parsed
    from an incoming stream and also generate messages that can be written to outgoing sinks. It
    supports 3 schemes for request and response termination, `content-length` encoding, `chunk` encoding
    and socket close.

* **Luminary**
  
  *A compiler for Lua*
  
  As an exercise in learning more about compilers, this project is a Lua front-end for
  LLVM.
  
## Education

**Eastern Michigan University**  
Bachelor of Science in Communications  
**Minneapolis Community Education**  
Associates in Software Development (Coursework)