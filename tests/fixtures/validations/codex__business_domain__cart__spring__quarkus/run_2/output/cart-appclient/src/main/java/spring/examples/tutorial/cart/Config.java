package spring.examples.tutorial.cart;

import java.io.IOException;
import java.io.InputStream;
import java.util.Properties;

public class Config {
    private static final Properties props = new Properties();
    static {
        try (InputStream is = Config.class.getClassLoader()
                .getResourceAsStream("application.properties")) {
            if (is != null) {
                props.load(is);
            }
        } catch (IOException e) {
            // ignore
        }
    }
    public static String get(String key) {
        return props.getProperty(key);
    }
}

