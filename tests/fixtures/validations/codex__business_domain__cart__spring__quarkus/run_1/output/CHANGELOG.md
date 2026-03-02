# Migration Changelog

## [2025-11-27T00:41:00Z] [info] Audit project and Spring usage
- Identified Spring Boot parent BOM, Spring web starter in `cart-app`, Spring context in `cart-common` and `cart-service`, and Spring client usage in `cart-appclient`.

## [2025-11-27T00:41:30Z] [info] Updated root BOM and plugins to Quarkus
- Replaced Spring Boot BOM with Quarkus BOM in `pom.xml`.
- Added `io.quarkus:quarkus-maven-plugin` to pluginManagement.

## [2025-11-27T00:42:30Z] [info] Replaced Spring dependencies with Quarkus
- `cart-common`: `spring-context` -> `quarkus-arc`.
- `cart-service`: `spring-context` -> `quarkus-arc`.
- `cart-app`: `spring-boot-starter-web` -> `quarkus-resteasy`, `quarkus-resteasy-jackson`, `quarkus-undertow`.
- `cart-appclient`: removed Spring Web starter, added `quarkus-arc` + `httpclient5`.

## [2025-11-27T00:43:15Z] [info] Refactored code to Quarkus APIs
- `CartServiceImpl`: `@Service` + Spring session scope -> `@Named @SessionScoped` (CDI); implements `Serializable`.
- `CartController`: Spring MVC annotations -> JAX-RS (`@Path`, `@GET/@POST/@DELETE`, `@QueryParam`) with `@Inject` Cart.
- `Application` (server): switched to JAX-RS `Application` class.
- `Application` (client): switched to `@QuarkusMain` with `QuarkusApplication` run method.
- `CartClient`: replaced Spring `RestTemplate` with Apache HttpClient 5; injected `app.cart.url` via MicroProfile Config.

## [2025-11-27T00:43:45Z] [info] Client cleanup and configs
- Removed Spring `SessionAwareRestTemplate` helper; replaced with Apache HttpClient 5 usage.
- Added Quarkus `application.properties` in `cart-app` and `cart-appclient`.

## [2025-11-27T00:44:10Z] [info] Build plugin configuration
- Added `quarkus-maven-plugin` to `cart-app` module.
- Added `quarkus-maven-plugin` to `cart-appclient` module.

## [2025-11-27T00:44:24Z] [error] Compilation failed due to restricted network
- Command: `mvn -q -Dmaven.repo.local=.m2repo clean package`
- Details: Maven could not resolve Quarkus BOM and dependencies because external network access is restricted.
- Impact: Build cannot complete without downloading Quarkus platform BOM and dependencies.
- Suggested Actions:
  - Re-run build with network enabled or a pre-populated local repo (`.m2repo`) containing Quarkus artifacts.
  - Alternatively, set up an internal Maven proxy/repository mirror accessible in this environment.
  - If offline compilation is required, vendor required jars into the repo using `systemPath` scope (not recommended for long-term).
