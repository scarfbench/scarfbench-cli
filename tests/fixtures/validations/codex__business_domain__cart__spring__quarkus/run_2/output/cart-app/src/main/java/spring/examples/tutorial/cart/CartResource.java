package spring.examples.tutorial.cart;

import jakarta.enterprise.context.RequestScoped;
import jakarta.inject.Inject;
import jakarta.servlet.http.HttpSession;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.Context;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import spring.examples.tutorial.cart.common.BookException;
import spring.examples.tutorial.cart.common.Cart;

import java.util.List;

@Path("/cart")
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
@RequestScoped
public class CartResource {

    @Inject
    Cart cart;

    @POST
    @Path("/initialize")
    public Response initialize(@QueryParam("person") String person,
                               @QueryParam("id") String id) throws BookException {
        if (id == null || id.isEmpty()) {
            cart.initialize(person);
        } else {
            cart.initialize(person, id);
        }
        return Response.noContent().build();
    }

    @POST
    @Path("/add")
    public Response addBook(@QueryParam("title") String title) {
        cart.addBook(title);
        return Response.noContent().build();
    }

    @DELETE
    @Path("/remove")
    public Response removeBook(@QueryParam("title") String title) throws BookException {
        cart.removeBook(title);
        return Response.noContent().build();
    }

    @GET
    @Path("/contents")
    public List<String> getContents() {
        return cart.getContents();
    }

    @POST
    @Path("/clear")
    public Response checkout(@Context HttpSession session) {
        cart.remove();
        if (session != null) {
            session.invalidate();
        }
        return Response.noContent().build();
    }
}

