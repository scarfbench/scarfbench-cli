# Migration Changelog

## [2025-11-27T01:03:24Z] [info] Initialized migration log
- Created `CHANGELOG.md` to track Spring -> Quarkus migration steps.

## [2025-11-27T01:03:40Z] [info] Updated Maven dependencies to Quarkus
- Removed Spring Boot parent and starters.
- Added Quarkus BOM and dependencies: `quarkus-resteasy`, `quarkus-resteasy-jackson`, `quarkus-arc`.
- Added Quarkus Maven plugin and compiler/surefire plugins.

## [2025-11-27T01:03:50Z] [info] Converted Spring components to Quarkus/CDI
- Replaced `@Service` with `@ApplicationScoped` in `ConverterService`.
- Replaced Spring MVC controller with JAX-RS resource using `@Path`, `@GET`, `@QueryParam` and CDI `@Inject`.
- Added JAX-RS `Application` subclass `QuarkusApplication` with `@ApplicationPath("/")`.

## [2025-11-27T01:03:55Z] [info] Updated configuration
- Replaced `spring.application.name` with `quarkus.application.name`.
- Set `quarkus.http.port=8080`.

## [2025-11-27T01:05:20Z] [info] Added static index page for Quarkus
- Added `src/main/resources/META-INF/resources/index.html` to serve a simple form at `/`.

## [2025-11-27T01:04:00Z] [info] Removed Spring Boot entrypoint
- Deleted `src/main/java/spring/examples/tutorial/converter/Application.java`.

## [2025-11-27T01:04:03Z] [error] Compilation failed due to offline repository restrictions
- Command: `mvn -q -Dmaven.repo.local=.m2repo clean package`
- Details: Maven cannot resolve Quarkus BOM and dependencies due to restricted network access. Errors include non-resolvable import POM and missing dependency versions (BOM import failed).
- Impact: Build cannot complete without local artifacts.
- Suggested Action: Provide a pre-populated local Maven repository (`.m2repo`) with Quarkus artifacts, or allow network access; alternatively pin explicit versions for dependencies and use a locally mirrored repository.

## [2025-11-27T01:04:10Z] [info] Minor controller cleanup
- Removed `@Context UriInfo` usage to simplify and avoid extra imports; output is static header.
## [2025-11-27T01:04:54Z] [error] Compilation failed using default local Maven repo
- Command: `mvn -q clean package`
- Details: Dependency resolution failed for `io.quarkus:quarkus-resteasy:3.15.1` due to network restrictions and missing local cache.
- Impact: Build remains incomplete.
- Suggested Action: Same as above; enable network access or provide cached artifacts.

## [2025-11-27T01:05:42Z] [error] Maven clean plugin resolution failed
- Command: `mvn -q -Dmaven.repo.local=.m2repo clean package`
- Details: `maven-clean-plugin` could not be resolved due to blocked network; central repository unreachable.
- Impact: Cannot execute Maven lifecycle in this environment without local plugin cache.
- Suggested Action: Pre-populate `.m2repo` with Maven plugins (clean, compiler, surefire) and Quarkus dependencies; or permit network access.

## [2025-11-27T01:06:10Z] [info] Restored Quarkus BOM and plugin in POM
- Reintroduced Quarkus BOM-managed dependencies and `quarkus-maven-plugin` to ensure proper Quarkus build when network/cache available.
