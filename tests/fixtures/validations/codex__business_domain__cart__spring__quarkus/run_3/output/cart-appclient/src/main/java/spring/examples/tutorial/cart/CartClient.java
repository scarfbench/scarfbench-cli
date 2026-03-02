package spring.examples.tutorial.cart;

import jakarta.enterprise.context.ApplicationScoped;
import org.eclipse.microprofile.config.inject.ConfigProperty;

import java.io.IOException;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;

@ApplicationScoped
public class CartClient {

    @ConfigProperty(name = "app.cart.url")
    String baseUrl;

    public void doCartOperations() throws IOException {
        httpPost(baseUrl + "/initialize?person=" + encode("Duke d'Url") + "&id=123");

        httpPost(baseUrl + "/add?title=" + encode("Infinite Jest"));
        httpPost(baseUrl + "/add?title=" + encode("Bel Canto"));
        httpPost(baseUrl + "/add?title=" + encode("Kafka on the Shore"));

        httpGet(baseUrl + "/contents");

        System.out.println("Removing \"Gravity's Rainbow\" from cart.");
        httpDelete(baseUrl + "/remove?title=" + encode("Gravity's Rainbow"));

        httpPost(baseUrl + "/clear");
    }

    private void httpPost(String url) throws IOException {
        request(url, "POST");
    }

    private void httpGet(String url) throws IOException {
        request(url, "GET");
    }

    private void httpDelete(String url) throws IOException {
        request(url, "DELETE");
    }

    private void request(String urlStr, String method) throws IOException {
        URL url = new URL(urlStr);
        HttpURLConnection conn = (HttpURLConnection) url.openConnection();
        conn.setRequestMethod(method);
        conn.setDoInput(true);
        int code = conn.getResponseCode();
        System.out.println(method + " " + urlStr + " -> " + code);
        try (InputStream is = (code >= 400 ? conn.getErrorStream() : conn.getInputStream())) {
            if (is != null) {
                byte[] buf = is.readAllBytes();
                String body = new String(buf, StandardCharsets.UTF_8);
                if (!body.isEmpty()) {
                    System.out.println("Response: " + body);
                }
            }
        }
        conn.disconnect();
    }

    private String encode(String s) {
        return s.replace(" ", "%20");
    }
}
