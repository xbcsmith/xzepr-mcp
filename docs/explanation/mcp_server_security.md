# Agents and MCP Servers: Are the electric sheep safe?

We have a new AI attack service. MCP servers are everywhere, and they are the new attack surface. Can the MCP server help protect the electric sheep from rogue agents and bad actors, or are they just another way to attack them? Agents are already being used to automate the software development lifecycle (SDLC), but they also introduce new risks. This talk explores the new attack surface created by MCP servers and agentic AI, focusing on potential vulnerabilities and mitigation strategies. We will discuss how agentic AI can enhance the SDLC while also addressing the security risks it introduces. The talk will cover the role of MCP servers in managing these risks and provide strategies for securing them against potential attacks.

Attendee Takeaways

Answers for the following questions:
- What does Agentic AI in the SDLC look like?
- What Security risk do Agentic AI bring to the SDLC?
- How can MCP servers help with Supply Chain Security?
- What are the risks of using MCP servers?
- What are strategies to mitigate attacks on MCP servers?


## Agentic AI in the SDLC

These are AI systems capable of taking high-level instructions and autonomously executing multi-step processes across different phases of the SDLC.

### Beyond Augmentation

Agentic AI moves beyond simply assisting developers (like code completion) to actively managing and executing tasks within the SDLC.

### Dynamic and Adaptive

Unlike traditional automation, agentic AI can dynamically adjust to changing contexts, user input, and environmental factors.

## New AI attack Surface

Supply Chain Attacks

Prompt Injection

Overreliance

Excessive Agency

Confabulation

Sensitive Data Leaks

Data Poisoning

Unchecked Output


## MCP Servers

What is an MCP server?

An MCP (Model Context Protocol) server is a standard server that exposes a contract-driven interface (JSON‑RPC over HTTP or stdio) so AI agents can discover, describe, and invoke external tools in a consistent, machine-readable way.

- **Model Context Protocol** - Open standard introduced November 2024
- **JSON-RPC interface** over HTTP/stdio for AI-tool integration
- **Standardized way** for AI agents to discover and interact with external tools
- **No custom plugins** required - unified contract-driven interface

AI Assistant (Client) <---> MCP Server <---> Your Tools/Data



## The laws of secure software have not changed

Security Amnesia

I mean eventually we all build something to run arbitrary commands in our pipelines... Passwordless SSH anyone? I am looking at you Ansible.

The rush to create AI solutions the industry seems to have forgotten application security 101. 

This regression in security practices has reintroduced attacks that we shouldn't see in 2025.

Remote Code Execution (#RCE) vulnerabilities, particularly from command injection, are the most prevalent.

Security 101

- Sanitize Input
- Need to Know
- Least Privileged
- Protect the Data
- Separate Concerns
- Check Results
 
## Not So Secure by Design

The Model Context Protocol (MCP) was designed primarily for functionality rather than security, creating fundamental vulnerabilities that cannot be easily patched in implementations:

- Session IDs in URLs: The protocol specification mandates session identifiers in URLs (GET /messages/?sessionId=UUID), which fundamentally violates security best practices. This design exposes sensitive identifiers in logs and a session could be hijacked by an attacker.
- Lack of Authentication Standards: The protocol provides minimal guidance on authentication, leading to inconsistent and often weak security implementations.
- Missing Integrity Controls: The protocol lacks required message signing or verification mechanisms, allowing message tampering.


MCP has design flaws that make security hard:

- Session IDs in URLs: GET /messages/?sessionId=123 – This breaks basic security rules
- No message signing: Can't verify if messages were tampered with
- Weak auth guidance: Protocol doesn't enforce authentication standards
- Trust model assumes good actors: No protection against malicious servers

## MCP Servers recurring security problems 

Vulnerabilities in MCP Servers

- Local
- Remote

Security challenges in MCP

MCP servers and tools face some recurring security problems:

- Broad access tokens: Many servers rely on long-lived static tokens. If leaked, these can be used to access any tool without restriction.
- No tenant isolation: In multi-user or multi-organization setups, all users share the same tool space. This can lead to accidental or malicious cross-access.
- Missing/Misconfigured rate limits: Without request throttling, AI agents can unintentionally overwhelm MCP servers by triggering tools much faster than human users would. This makes rate limiting a critical consideration in MCP setups.
- Unverified tool updates: Tools can be redefined silently without any notification to the client, changing behavior after approval.
- Lack of auditing: Users and developers can't always see what changed, when, or by whom.

The Universal Attack Surface

An important aspect often overlooked: MCP servers can be called by anyone, not just LLMs. While LLMs typically show what they're going to do with "plan" and "act" phases, a malicious attacker has no such transparency. This creates an expanded attack surface that many developers haven't properly secured.

Treat MCP Servers like any other server in your pipeline.

## Attacks and Mitigation Strategies

### Confused Deputy Problem

Attackers can exploit MCP servers proxying other resource servers, creating "confused deputy" vulnerabilities.

#### Attack Description

When an MCP proxy server uses a static client ID to authenticate with a third-party authorization server that does not support dynamic client registration, the following attack becomes possible:

1.    A user authenticates normally through the MCP proxy server to access the third-party API
2.    During this flow, the third-party authorization server sets a cookie on the user agent indicating consent for the static client ID
3.    An attacker later sends the user a malicious link containing a crafted authorization request which contains a malicious redirect URI along with a new dynamically registered client ID
4.    When the user clicks the link, their browser still has the consent cookie from the previous legitimate request
5.    The third-party authorization server detects the cookie and skips the consent screen
6.    The MCP authorization code is redirected to the attacker's server (specified in the crafted redirect_uri during dynamic client registration)
7.    The attacker exchanges the stolen authorization code for access tokens for the MCP server without the user's explicit approval
8.    Attacker now has access to the third-party API as the compromised user

#### Mitigation

MCP proxy servers using static client IDs MUST obtain user consent for each dynamically registered client before forwarding to third-party authorization servers (which may require additional consent).


### Token Passthrough

"Token passthrough" is an anti-pattern where an MCP server accepts tokens from an MCP client without validating that the tokens were properly issued to the MCP server and "passing them through" to the downstream API.


#### Risks

Token passthrough is explicitly forbidden in the authorization specification as it introduces a number of security risks, that include:

Security Control Circumvention

The MCP Server or downstream APIs might implement important security controls like rate limiting, request validation, or traffic monitoring, that depend on the token audience or other credential constraints. If clients can obtain and use tokens directly with the downstream APIs without the MCP server validating them properly or ensuring that the tokens are issued for the right service, they bypass these controls.

Accountability and Audit Trail Issues

The MCP Server will be unable to identify or distinguish between MCP Clients when clients are calling with an upstream-issued access token which may be opaque to the MCP Server. The downstream Resource Server's logs may show requests that appear to come from a different source with a different identity, rather than the MCP server that is actually forwarding the tokens.Both factors make incident investigation, controls, and auditing more difficult. If the MCP Server passes tokens without validating their claims (e.g., roles, privileges, or audience) or other metadata, a malicious actor in possession of a stolen token can use the server as a proxy for data exfiltration.
    
Trust Boundary Issues

The downstream Resource Server grants trust to specific entities. This trust might include assumptions about origin or client behavior patterns. Breaking this trust boundary could lead to unexpected issues. If the token is accepted by multiple services without proper validation, an attacker compromising one service can use the token to access other connected services.

Future Compatibility Risk

Even if an MCP Server starts as a "pure proxy" today, it might need to add security controls later. Starting with proper token audience separation makes it easier to evolve the security model.

​
#### Mitigation

MCP servers MUST NOT accept any tokens that were not explicitly issued for the MCP server.

### Session Hijacking

Session hijacking is an attack vector where a client is provided a session ID by the server, and an unauthorized party is able to obtain and use that same session ID to impersonate the original client and perform unauthorized actions on their behalf.

#### Attack Description

When you have multiple stateful HTTP servers that handle MCP requests, the following attack vectors are possible: Session Hijack Prompt Injection

1. The client connects to Server A and receives a session ID.
2. The attacker obtains an existing session ID and sends a malicious event to Server B with said session ID. When a server supports redelivery/resumable streams, deliberately terminating the request before receiving the response could lead to it being resumed by the original client via the GET request for server sent events. If a particular server initiates server sent events as a consequence of a tool call such as a notifications/tools/list_changed, where it is possible to affect the tools that are offered by the server, a client could end up with tools that they were not aware were enabled.
3. Server B enqueues the event (associated with session ID) into a shared queue.
4. Server A polls the queue for events using the session ID and retrieves the malicious payload.
5. Server A sends the malicious payload to the client as an asynchronous or resumed response.
6. The client receives and acts on the malicious payload, leading to potential compromise.

Session Hijack Impersonation

1. The MCP client authenticates with the MCP server, creating a persistent session ID.
2. The attacker obtains the session ID.
3. The attacker makes calls to the MCP server using the session ID.
4. MCP server does not check for additional authorization and treats the attacker as a legitimate user, allowing unauthorized access or actions.

​
#### Mitigation

To prevent session hijacking and event injection attacks, the following mitigations should be implemented: MCP servers that implement authorization MUST verify all inbound requests. MCP Servers MUST NOT use sessions for authentication. MCP servers MUST use secure, non-deterministic session IDs. Generated session IDs (e.g., UUIDs) SHOULD use secure random number generators. Avoid predictable or sequential session identifiers that could be guessed by an attacker. Rotating or expiring session IDs can also reduce the risk. MCP servers SHOULD bind session IDs to user-specific information. When storing or transmitting session-related data (e.g., in a queue), combine the session ID with information unique to the authorized user, such as their internal user ID. Use a key format like <user_id>:<session_id>. This ensures that even if an attacker guesses a session ID, they cannot impersonate another user as the user ID is derived from the user token and not provided by the client. MCP servers can optionally leverage additional unique identifiers.

### Toxic Flow Attacks

### Tool Poisoning Attacks

#### Attack Description

Tool Poisoning Attacks: Malicious instructions are hidden in tool descriptions to cause unauthorized actions like data exfiltration. A seemingly innocent tool could contain hidden instructions to read sensitive files.

### Mitigation

Clear UI Patterns: Tool descriptions should be clearly visible to users, clearly distinguishing between user-visible and AI-visible instructions. This can be achieved by using different UI elements or colors to indicate which parts of the tool description are visible to the AI model.

Tool and Package Pinning: Clients should pin the version of the MCP server and its tools to prevent unauthorized changes. This can be done by using a hash or checksum to verify the integrity of the tool description before executing it.

Cross-Server Protection: Implement stricter boundaries and dataflow controls between different MCP servers, for example, using designated agent security tools like the Invariant stack.


### Rug Pulls

Silent Redefinition: Tools change their behavior silently after initial approval to perform malicious actions, such as rerouting API keys. Most clients don't notify users of these changes.

### Cross-Server Tool Shadowing

A malicious server can intercept calls intended for a trusted server, exploiting the LLM's trust (Confused Deputy).

### Insecure Credential Storage

Storing API keys in plaintext makes them vulnerable to theft. This is observed in tools for GitLab, Postgres, Google Maps, and third-party connectors.

### Line Jumping Attack

Prompt injections can be made via tool descriptions to bypass security measures before user approval or tool invocation. This exploits the fact that clients update context with tool descriptions upon connection.


## The Lethal Trifecta

The lethal trifecta for AI agents: private data, untrusted content, and external communication

If you are a user of LLM systems that use tools (you can call them "AI agents" if you like) it is critically important that you understand the risk of combining tools with the following three characteristics. Failing to understand this can let an attacker steal your data.

The lethal trifecta of capabilities is:

    Access to your private data,one of the most common purposes of tools in the first place!
    Exposure to untrusted content,any mechanism by which text (or images) controlled by a malicious attacker could become available to your LLM
    The ability to externally communicate in a way that could be used to steal your data (I often call this "exfiltration" but I'm not confident that term is widely understood.)

If your agent combines these three features, an attacker can easily trick it into accessing your private data and sending it to that attacker.

The recently discovered GitHub MCP exploit provides an example where one MCP mixed all three patterns in a single tool. That MCP can read issues in public issues that could have been filed by an attacker, access information in private repos and create pull requests in a way that exfiltrates that private data.

- Private Data- Tools that can read secrets, repos, files
- Untrusted Content- Attacker-controlled input (issues, PRs, web content)
- External Communication- Any channel that can exfiltrate data (PRs, webhooks, email)

Why it matters

- If an agent/tool has all three, an attacker can trick it into reading private data and sending it out.

Mitigations (high level)

- Enforce least privilege: separate tools that read private data from tools that can publish externally
- Treat untrusted inputs as hostile: validate, sanitize, and isolate before exposing to agents
- Block or tightly control outbound channels from tools that access sensitive data
- Monitor for suspicious patterns: unexpected reads + outbound actions trigger alerts

Notes

- The danger comes from the combination: each capability alone is manageable; together they enable straightforward exfiltration.
- Preventive design (isolation, policy) and runtime detection (alerts, rate limits) are both required.

## Links

- [https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices](https://modelcontextprotocol.io/specification/2025-06-18/basic/security_best_practices)
- [https://www.infracloud.io/blogs/securing-mcp-servers/](https://www.infracloud.io/blogs/securing-mcp-servers)
- [https://invariantlabs.ai/blog/mcp-github-vulnerability](https://invariantlabs.ai/blog/mcp-github-vulnerability)
- [https://noailabs.medium.com/toxic-agents-the-perfect-weapon-for-attackers-130ba4ca847c](https://noailabs.medium.com/toxic-agents-the-perfect-weapon-for-attackers-130ba4ca847c)
- [https://invariantlabs.ai/blog/toxic-flow-analysis](https://invariantlabs.ai/blog/toxic-flow-analysis)
- [https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/)
