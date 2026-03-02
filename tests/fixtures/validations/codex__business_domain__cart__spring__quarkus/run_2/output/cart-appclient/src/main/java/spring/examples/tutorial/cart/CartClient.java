package spring.examples.tutorial.cart;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

public class CartClient {

    private final String baseUrl;

    public CartClient(String baseUrl) {
        this.baseUrl = baseUrl;
    }

    public void doCartOperations() throws Exception {
        HttpClient client = HttpClient.newBuilder().build();

        HttpRequest init = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/initialize?person=" + urlEncode("Duke d'Url") + "&id=123"))
                .POST(HttpRequest.BodyPublishers.noBody())
                .build();
        client.send(init, HttpResponse.BodyHandlers.discarding());

        for (String title : List.of("Infinite Jest", "Bel Canto", "Kafka on the Shore")) {
            HttpRequest add = HttpRequest.newBuilder()
                    .uri(URI.create(baseUrl + "/add?title=" + urlEncode(title)))
                    .POST(HttpRequest.BodyPublishers.noBody())
                    .build();
            client.send(add, HttpResponse.BodyHandlers.discarding());
        }

        HttpRequest get = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/contents"))
                .GET()
                .build();
        String body = client.send(get, HttpResponse.BodyHandlers.ofString()).body();
        for (String t : parseJsonArray(body)) {
            System.out.println("Retrieving book title from cart: " + t);
        }

        System.out.println("Removing \"Gravity's Rainbow\" from cart.");
        HttpRequest del = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/remove?title=" + urlEncode("Gravity's Rainbow")))
                .DELETE()
                .build();
        client.send(del, HttpResponse.BodyHandlers.discarding());

        HttpRequest clear = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/clear"))
                .POST(HttpRequest.BodyPublishers.noBody())
                .build();
        client.send(clear, HttpResponse.BodyHandlers.discarding());
    }

    private static String urlEncode(String s) {
        return java.net.URLEncoder.encode(s, StandardCharsets.UTF_8);
    }

    private static List<String> parseJsonArray(String json) {
        // Very basic JSON array of strings parser: ["a","b"] -> [a,b]
        List<String> out = new ArrayList<>();
        String trimmed = json.trim();
        if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
            String inner = trimmed.substring(1, trimmed.length() - 1).trim();
            if (!inner.isEmpty()) {
                for (String part : inner.split(",")) {
                    String s = part.trim();
                    if (s.startsWith("\"") && s.endsWith("\"")) {
                        s = s.substring(1, s.length() - 1);
                    }
                    out.add(s.replace("\\\"", "\""));
                }
            }
        }
        return out;
    }
}
