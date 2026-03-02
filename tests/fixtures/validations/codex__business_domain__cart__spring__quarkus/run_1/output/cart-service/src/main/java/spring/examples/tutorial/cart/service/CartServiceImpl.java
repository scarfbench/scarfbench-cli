package spring.examples.tutorial.cart.service;

import jakarta.enterprise.context.SessionScoped;
import jakarta.inject.Named;
import spring.examples.tutorial.cart.common.BookException;
import spring.examples.tutorial.cart.common.Cart;
import spring.examples.tutorial.cart.common.IdVerifier;
import java.util.ArrayList;
import java.util.List;
import java.io.Serializable;

@Named
@SessionScoped
public class CartServiceImpl implements Cart, Serializable {

    private String customerId;
    private String customerName;
    private List<String> contents;

    @Override
    public void initialize(String person) throws BookException {
        if (person == null) {
            throw new BookException("Null person not allowed.");
        }
        customerName = person;
        customerId = "0";
        contents = new ArrayList<>();
    }

    @Override
    public void initialize(String person, String id) throws BookException {
        if (person == null) {
            throw new BookException("Null person not allowed.");
        }
        customerName = person;

        IdVerifier idChecker = new IdVerifier();
        if (idChecker.validate(id)) {
            customerId = id;
        } else {
            throw new BookException("Invalid id: " + id);
        }

        contents = new ArrayList<>();
    }

    @Override
    public void addBook(String title) {
        contents.add(title);
    }

    @Override
    public void removeBook(String title) throws BookException {
        boolean result = contents.remove(title);
        if (!result) {
            throw new BookException("\"" + title + "\" not in cart.");
        }
    }

    @Override
    public List<String> getContents() {
        return contents;
    }

    @Override
    public void remove() {
        contents = null;
    }
}
