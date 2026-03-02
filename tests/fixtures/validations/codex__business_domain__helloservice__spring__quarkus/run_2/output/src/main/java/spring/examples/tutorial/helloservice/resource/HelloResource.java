package spring.examples.tutorial.helloservice.resource;

import jakarta.inject.Inject;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.MediaType;
import spring.examples.tutorial.helloservice.service.HelloService;

@Path("/hello")
public class HelloResource {

    @Inject
    HelloService helloService;

    @GET
    @Produces(MediaType.TEXT_PLAIN)
    public String sayHello(@QueryParam("name") String name) {
        return helloService.sayHello(name);
    }
}

