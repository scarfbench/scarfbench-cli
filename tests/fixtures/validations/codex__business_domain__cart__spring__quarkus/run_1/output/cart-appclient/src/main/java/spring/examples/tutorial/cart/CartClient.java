package spring.examples.tutorial.cart;

import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import org.eclipse.microprofile.config.inject.ConfigProperty;
import org.apache.hc.client5.http.classic.methods.HttpDelete;
import org.apache.hc.client5.http.classic.methods.HttpGet;
import org.apache.hc.client5.http.classic.methods.HttpPost;
import org.apache.hc.client5.http.impl.classic.CloseableHttpClient;
import org.apache.hc.client5.http.impl.classic.HttpClients;
import org.apache.hc.core5.http.io.entity.StringEntity;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

@ApplicationScoped
public class CartClient {

    @ConfigProperty(name = "app.cart.url")
    String baseUrl;

    public void doCartOperations() throws Exception {
        try (CloseableHttpClient client = HttpClients.createDefault()) {
            HttpPost init = new HttpPost(baseUrl + "/initialize?person=" +
                    encode("Duke d'Url") + "&id=123");
            client.execute(init).close();

            client.execute(new HttpPost(baseUrl + "/add?title=" + encode("Infinite Jest"))).close();
            client.execute(new HttpPost(baseUrl + "/add?title=" + encode("Bel Canto"))).close();
            client.execute(new HttpPost(baseUrl + "/add?title=" + encode("Kafka on the Shore"))).close();

            HttpGet get = new HttpGet(baseUrl + "/contents");
            var response = client.execute(get);
            List<String> books = new ArrayList<>();
            try (var entity = response.getEntity();
                 var in = new BufferedReader(new InputStreamReader(entity.getContent()))) {
                String line;
                while ((line = in.readLine()) != null) {
                    books.add(line);
                }
            }
            books.forEach(title -> System.out.println("Retrieving book title from cart: " + title));

            System.out.println("Removing \"Gravity's Rainbow\" from cart.");
            client.execute(new HttpDelete(baseUrl + "/remove?title=" + encode("Gravity's Rainbow"))).close();
            client.execute(new HttpPost(baseUrl + "/clear")).close();
        }
    }

    private String encode(String s) {
        return s.replace(" ", "%20").replace("'", "%27");
    }
}
