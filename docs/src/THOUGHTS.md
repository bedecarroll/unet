# unet the simple network configuration system

This is a stream of conciousness that we need to change into a workable design.

- unet but the u is the symbol for micro
- rust built system
- we are very simple on purpose
- backends we should support:
  - CSVs for a demo mode
  - SQLite (preferred) as this will scale very high
  - Postgres but todo for later
- auth is simple:
  - no auth
  - basic username/password in our db
  - for advanced maybe some load balancer like thing but this is TBD and low pri
- client is rust CLI tool to talk to rust Server backend
- transport is probably HTTP(s)
- the server stores simple system data on network nodes
- we need to store things like:
  - node_name
  - domain_name
  - software_version
  - vendor
  - model
  - device_role
  - etc
- links are a first class citizen and its own table
  - this means we need to support tying interfaces between devices
  - need to support no Z endpoint for internet circuits
  - we need a link_role field
- we need the client and server to share a library for the following features:
  - policy engine
- We have an important feature, the concept of desired vs derived for data
  - the desired is what a user will populate the database with using the CLI
  - derived is our system using SNMP to reach out to the device on the
  management IP
  - we want full lifecycle, plan, implementing (as in not live device), live, decomm
  - we need a location table and it should use the same lifecycle states
  - the links and node tables also need lifecycle fields
- The policy engine, this is a key feature
  - We either need our own language or something like Apple PKL that we can create
  a match statement for
  - This allows us to do the following things to the JSON that our system produces:
    - update fields
    - add new fields in custom_data
  - the update allows us to define rules on the desired table
  - example:
    `node.vendor == 'juniper' and node.model =~ (contains) 'qfx' and node.model !~ (not contains) '32q'; software_version == '17.2'`
    This rule says, juniper devices that are qfx but not the 32q models must
  have a software_version of 17.2
  - this allows us to compare desired vs derived as derived will pull the
  version via SNMP and we can see the mismatch
  - this also allows us to control rollout
- we need to support MCP, Model Context Protocol so that LLMs can use our data
- the policy files should be pulled from git that the user can specify in
server config
- the server should apply the policy on a user configurable schedule
- our ORM should allow users to specify their own fields and apply to db schema
  - the "add new fields via policy" feature is to allow users to see if a field
  is useful before adding to db
  - these "virtual fields" should be clearly isolated in something like
  "custom_data" which is an empty dictionary by default for the JSON we send to
  client

## client

- client should also use the policy engine locally
- the client should be able to get live data from the server or be able to be
pointed to local policy files
- this allows operators to modify files locally to test changes (canary mode)
- we should support pushing canary policy files from client to server
  - the server will override this as per schedule but useful in scenarios when
  user needs temporary changes
- client should be able to render templates to generate device configurations
- use rust minijinja
- the full node information json is available to the templates
- policy on server and in client needs to support "applying" a template path to
a node
  - this is important as it allows us to break up templates
  - example:
    - template juniper/qfx/system.jinja is applied as per policy match language
    - template juniper/interfaces.jinja applied to all juniper
    - template cisco/interface.jinja applied to cisco
  - the path represents a physical file in the git repo
  - structure is entirely up to the user and the policy is what maps file to
  node
  - this ability to use policy is what makes it powerful to have different
  templates apply to different device for any user defined reason
  - these files can also be "canaried" so that users can render locally and
  test changes
- to prevent overlapping templates and to allow us to support diffing our
rendered templates and a live configuration
  - build a "template match" feature to put at the top of template files
  - we will need to parse network configuration hierarchy of different vendors
  (this code needs to be shared and modular)
  - match syntax should look something like `interfaces ge-.*||.*||.*` this
  matches all interfaces that start with `ge-` and
  gets all descendent configuration lines
  - another example would be `system||.*||.*` on juniper systems where the
  `system {` is a top level
  - another would be `system||dhcp||options .*||.*||.*` where we match all
  lines ONLY after we reach the depth of system, dhcp options
  - the `||.*||.*` is a special match everything else at infinite depth code.
  We could use something else
  - if user doesn't want to break up templates and wants 1 large one they could
  do `.*||.*||.*` which matches all lines but this would be bad practice as it
  would be hard to manage and isolate changes
- diff and render are two important features
- diff allows us to compare template to device config, we need to use the match feature
  - user can specify something like `diff -t interface -o device_output.txt`
  or even `diff -o device_output.txt`
  - `-t` would be a template name that policy has applied to device
  - if no template is applied we render the whole thing which template match
  helps with "snippeting" the config specified with `-o`
  - we MUST support the include capability in jinja so that user can template
  their entire config while still maintaining separation
- debug output is important to understand policy and template inner workings
- we are trying to be simple and POWERFUL particularly in existing environments
  - we allow users to have differences between desired/derived
  - we allow users to be creative to control things like rollouts with policy language
  - we allow users to progressively template network environments
  - we support full network/device/site (location) lifecycle
  - we support best practices with respect to code by allowing users to test and
  capture output of potential changes locally and attaching to PRs or commits
  as test plans
- in the future it would be great to pull configurations from devices OR allow
  users to pull from a git repo of configs
