# =============================================================================
# Star UI — Railway Deployment (Static Export)
# =============================================================================
FROM node:20-alpine AS builder

WORKDIR /app

COPY ui/package.json ui/package-lock.json* ./
RUN npm ci --legacy-peer-deps

COPY ui/ ./

RUN npm run build

# =============================================================================
# Runtime stage - nginx to serve static files
# =============================================================================
FROM nginx:alpine

COPY --from=builder /app/out /usr/share/nginx/html

# Configure nginx for SPA routing
RUN echo 'server { \
    listen 8080; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    \
    # Cache static assets \
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ { \
        expires 1y; \
        add_header Cache-Control "public, immutable"; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

CMD ["nginx", "-g", "daemon off;"]
