package spring.examples.tutorial.cart;

public class Application {
    public static void main(String[] args) throws Exception {
        CartClient cartClient = new CartClient(Config.get("app.cart.url"));
        try {
            cartClient.doCartOperations();
        } catch (Exception ex) {
            System.err.println("Caught an exception: " + ex.getMessage());
        }
    }
}
