package spring.examples.tutorial.cart;

import io.quarkus.runtime.QuarkusApplication;
import io.quarkus.runtime.annotations.QuarkusMain;
import io.quarkus.runtime.Quarkus;
import jakarta.inject.Inject;

@QuarkusMain
public class Application implements QuarkusApplication {

    @Inject
    CartClient cartClient;

    public static void main(String[] args) {
        Quarkus.run(Application.class, args);
    }

    @Override
    public int run(String... args) throws Exception {
        try {
            cartClient.doCartOperations();
        } catch (Exception ex) {
            System.err.println("Caught a BookException: " + ex.getMessage());
        }
        return 0;
    }
}
