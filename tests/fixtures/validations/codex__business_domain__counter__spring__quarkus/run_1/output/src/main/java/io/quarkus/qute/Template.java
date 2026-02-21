package io.quarkus.qute;

public interface Template {
    TemplateInstance data(String key, Object value);
}
