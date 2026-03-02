package spring.examples.tutorial.counter.controller;

import jakarta.inject.Inject;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import io.quarkus.qute.Template;
import io.quarkus.qute.Location;
import io.quarkus.qute.TemplateInstance;
import spring.examples.tutorial.counter.service.CounterService;

@Path("/")
public class CountController {

    @Inject
    CounterService counterService;

    @Inject
    @Location("index.html")
    Template index;

    @GET
    @Produces(MediaType.TEXT_HTML)
    public TemplateInstance index() {
        int hitCount = counterService.getHits();
        return index.data("hitCount", hitCount);
    }
}
