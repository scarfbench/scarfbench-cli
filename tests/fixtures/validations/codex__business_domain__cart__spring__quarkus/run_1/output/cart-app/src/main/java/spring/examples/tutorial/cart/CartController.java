package spring.examples.tutorial.cart;

import jakarta.inject.Inject;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Context;
import jakarta.servlet.http.HttpServletRequest;
import spring.examples.tutorial.cart.common.BookException;
import spring.examples.tutorial.cart.common.Cart;
import java.util.List;

@Path("/cart")
public class CartController {

    @Inject
    Cart cart;

    @POST
    @Path("/initialize")
    public void initialize(@QueryParam("person") String person,
                           @QueryParam("id") String id) throws BookException {
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
    public void checkout(@Context HttpServletRequest request) {
        cart.remove();
        request.getSession().invalidate();
    }
}
