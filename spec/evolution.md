# Schema Evolution Guidelines


1. **Adding optional fields:** Allowed in patch versions
2. **Adding new domains:** Allowed in minor versions
3. **Deprecating fields:** Mark with `deprecated: true`, remove in next major version
4. **Renaming fields:** Requires major version; provide migration guide
5. **Changing types:** Requires major version
6. **Extensions:** Vendor extensions can evolve independently of core schema

---

