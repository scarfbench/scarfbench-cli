package spring.examples.tutorial.cart;

import jakarta.enterprise.context.RequestScoped;
import jakarta.inject.Inject;
import jakarta.servlet.http.HttpSession;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.MediaType;
import spring.examples.tutorial.cart.common.BookException;
import spring.examples.tutorial.cart.common.Cart;
import java.util.List;

@Path("/cart")
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
@RequestScoped
public class CartController {

    @Inject
    Cart cart;

    @POST
    @Path("/initialize")
    public void initialize(@QueryParam("person") String person,
                           @QueryParam("id") String id)
            throws BookException {
        if (id == null) {
            cart.initialize(person);
        } else {
            cart.initialize(person, id);
        }
    }

    @POST
    @Path("/add")
    public void addBook(@QueryParam("title") String title) {
        cart.addBook(title);
    }

    @DELETE
    @Path("/remove")
    public void removeBook(@QueryParam("title") String title) throws BookException {
        cart.removeBook(title);
    }

    @GET
    @Path("/contents")
    public List<String> getContents() {
        return cart.getContents();
    }

    @POST
    @Path("/clear")
    public void checkout(@Inject HttpSession session) {
        cart.remove();
        session.invalidate();
    }

}
