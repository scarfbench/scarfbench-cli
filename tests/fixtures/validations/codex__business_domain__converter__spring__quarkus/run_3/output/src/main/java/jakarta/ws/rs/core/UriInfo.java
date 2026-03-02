package jakarta.ws.rs.core;

public class UriInfo {
    private final String path;

    public UriInfo(String path) {
        this.path = path;
    }

    public String getPath() {
        return path;
    }
}

