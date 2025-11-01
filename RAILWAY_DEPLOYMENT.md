# Railway Deployment Guide

## Quick Deploy via GitHub (Recommended)

The easiest way to deploy your Flow API to Railway:

### 1. Deploy from GitHub

1. Go to [Railway Dashboard](https://railway.app/dashboard)
2. Click **"New Project"**
3. Select **"Deploy from GitHub repo"**
4. Authorize Railway to access your GitHub account
5. Select the **`MerlijnW70/flow`** repository
6. Railway will automatically detect the Dockerfile and deploy

### 2. Add PostgreSQL Database

1. In your Railway project dashboard
2. Click **"New"** → **"Database"** → **"Add PostgreSQL"**
3. Railway automatically creates a `DATABASE_URL` environment variable

### 3. Configure Environment Variables

Click on your service → **"Variables"** tab → Add the following:

```bash
# Required
JWT_SECRET=your-super-secret-key-minimum-32-characters-long
ENVIRONMENT=production
RUST_LOG=info,vibe_api=debug

# Optional (if using AI features)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Optional (if using S3 storage)
S3_BUCKET=your-bucket-name
S3_REGION=us-east-1
S3_ACCESS_KEY=your-access-key
S3_SECRET_KEY=your-secret-key
```

**Note**: Railway automatically provides:
- `DATABASE_URL` (from PostgreSQL addon)
- `PORT` (Railway assigns dynamically)

### 4. Deploy

Railway will automatically:
- ✅ Build using the Dockerfile
- ✅ Run migrations (if configured)
- ✅ Deploy to production
- ✅ Provide HTTPS endpoint
- ✅ Monitor health checks

### 5. Access Your API

Once deployed, Railway provides a public URL like:
```
https://flow-production-xxxx.up.railway.app
```

Test your endpoints:
- Health: `https://your-app.up.railway.app/health`
- Metrics: `https://your-app.up.railway.app/metrics`
- Swagger: `https://your-app.up.railway.app/swagger-ui`

---

## Alternative: Deploy via Railway CLI

### Prerequisites
```bash
npm install -g @railway/cli
```

### Step-by-Step

1. **Login to Railway**
   ```bash
   railway login
   ```

2. **Initialize Project**
   ```bash
   railway init
   ```
   Select "Create new project" and give it a name.

3. **Link to Repository**
   ```bash
   railway link
   ```

4. **Add PostgreSQL**
   ```bash
   railway add --database postgres
   ```

5. **Set Environment Variables**
   ```bash
   railway variables set JWT_SECRET="your-secret-key-here"
   railway variables set ENVIRONMENT="production"
   railway variables set RUST_LOG="info,vibe_api=debug"
   ```

6. **Deploy**
   ```bash
   railway up
   ```

7. **View Logs**
   ```bash
   railway logs
   ```

8. **Open in Browser**
   ```bash
   railway open
   ```

---

## Configuration Files

### railway.toml
```toml
[build]
builder = "DOCKERFILE"
dockerfilePath = "apps/api/Dockerfile"

[deploy]
startCommand = "/app/vibe-api"
healthcheckPath = "/health"
healthcheckTimeout = 100
restartPolicyType = "ON_FAILURE"
restartPolicyMaxRetries = 10
```

### Dockerfile
The multi-stage Dockerfile at `apps/api/Dockerfile`:
- **Stage 1**: Builds Rust binary with optimizations
- **Stage 2**: Minimal runtime image with security best practices
- **Size**: ~100MB final image
- **Build time**: ~5-8 minutes (first build)

---

## Database Migrations

### Option 1: Run manually via Railway CLI
```bash
railway run cargo sqlx migrate run
```

### Option 2: Add to Dockerfile
Uncomment the migration step in Dockerfile before CMD:
```dockerfile
# Run migrations (optional - uncomment if needed)
# RUN /app/vibe-api migrate

CMD ["/app/vibe-api"]
```

### Option 3: Run from Railway shell
```bash
railway shell
cd /app
./vibe-api migrate
```

---

## Monitoring & Debugging

### View Logs
```bash
railway logs --follow
```

### Check Deployment Status
```bash
railway status
```

### Access Metrics
```
https://your-app.up.railway.app/metrics
```

### Health Check
```
https://your-app.up.railway.app/health
```
Should return:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime_seconds": 123
  }
}
```

---

## Scaling & Performance

### Horizontal Scaling
Railway supports horizontal scaling:
1. Go to your service settings
2. Enable "Horizontal Scaling"
3. Set min/max replicas

### Vertical Scaling
Upgrade resources:
1. Go to service settings
2. Select "Resources"
3. Choose larger instance size

### Estimated Costs
- **Hobby Plan**: $5/month (500 hours, 512MB RAM, 1GB disk)
- **Pro Plan**: $20/month (unlimited hours, configurable resources)
- **PostgreSQL**: Included in plans

---

## Troubleshooting

### Build Fails

**Issue**: "Out of memory during Rust compilation"
**Solution**: Upgrade to larger build instance or enable swap:
```dockerfile
# Add before cargo build
ENV CARGO_BUILD_JOBS=2
```

### Database Connection Issues

**Issue**: "Failed to connect to database"
**Solution**: Verify DATABASE_URL is set:
```bash
railway variables
```

### Health Check Failing

**Issue**: Railway shows "Unhealthy"
**Solution**: Check logs:
```bash
railway logs
```
Verify `/health` endpoint returns 200 OK.

### Port Binding Issues

**Issue**: "Address already in use"
**Solution**: Ensure your app uses Railway's `PORT` environment variable:
```rust
let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
```

---

## Security Best Practices

### Environment Variables
✅ **DO**:
- Store secrets in Railway environment variables
- Use strong JWT_SECRET (32+ characters)
- Enable DATABASE_URL encryption

❌ **DON'T**:
- Commit secrets to Git
- Use default/weak passwords
- Expose debug endpoints in production

### Database
✅ **DO**:
- Use Railway's managed PostgreSQL
- Enable SSL connections
- Regular backups (automatic with Railway)

### HTTPS
✅ Railway provides automatic HTTPS for all deployments

---

## Post-Deployment Checklist

- [ ] API is accessible via Railway URL
- [ ] Health check returns 200 OK
- [ ] Database connection successful
- [ ] All environment variables set
- [ ] Swagger UI accessible
- [ ] Metrics endpoint working
- [ ] Test authentication endpoints
- [ ] Review logs for errors
- [ ] Set up monitoring/alerts
- [ ] Configure custom domain (optional)

---

## Custom Domain (Optional)

1. Go to Railway dashboard
2. Select your service
3. Click "Settings" → "Domains"
4. Click "Add Custom Domain"
5. Enter your domain (e.g., `api.yourdomain.com`)
6. Add CNAME record to your DNS:
   ```
   CNAME api.yourdomain.com → your-app.up.railway.app
   ```

Railway automatically provisions SSL certificate.

---

## Continuous Deployment

Railway automatically redeploys when you push to GitHub:

```bash
git add .
git commit -m "Update feature"
git push origin main
```

Railway will:
1. Detect the push
2. Build new Docker image
3. Run health checks
4. Deploy with zero downtime
5. Rollback if health checks fail

---

## Useful Commands

```bash
# View all projects
railway list

# Switch project
railway link

# Environment variables
railway variables
railway variables set KEY=value
railway variables delete KEY

# Logs
railway logs
railway logs --follow

# Open in browser
railway open

# SSH into container
railway shell

# Status
railway status

# Redeploy
railway up
```

---

## Support

- **Railway Docs**: https://docs.railway.app
- **Railway Discord**: https://discord.gg/railway
- **Project Issues**: https://github.com/MerlijnW70/flow/issues

---

## Summary

Your Flow API is production-ready with:
- ✅ 219 integration tests
- ✅ Multi-stage optimized Dockerfile
- ✅ Health checks & metrics
- ✅ PostgreSQL database
- ✅ Automatic HTTPS
- ✅ Zero-downtime deployments
- ✅ Swagger UI documentation

**Estimated deployment time**: 5-10 minutes (first deployment)

Happy deploying! 🚀
