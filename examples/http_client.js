#!/usr/bin/env node

/**
 * PM4Py REST API Client Examples (JavaScript/Node.js)
 *
 * Demonstrates how to use the PM4Py REST API from JavaScript/Node.js applications.
 * This example covers:
 * - Authentication
 * - Process discovery
 * - Conformance checking
 * - Log statistics
 * - Error handling and rate limiting
 *
 * Installation:
 *   npm install node-fetch
 *
 * Usage:
 *   export PM4PY_API_KEY="your-api-key"
 *   node examples/http_client.js
 *
 * Or with npm scripts in package.json:
 *   "scripts": { "api-examples": "node examples/http_client.js" }
 *   npm run api-examples
 */

const fetch = require("node-fetch");

/**
 * PM4Py REST API Client
 */
class PM4PyClient {
  constructor(apiKey, baseUrl = "http://localhost:8080/api/v1") {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
    this.rateLimitRemaining = 10000;
    this.rateLimitReset = null;
  }

  /**
   * Update rate limit information from response headers
   */
  updateRateLimitInfo(response) {
    const remaining = response.headers.get("X-RateLimit-Remaining");
    const reset = response.headers.get("X-RateLimit-Reset");

    if (remaining) {
      this.rateLimitRemaining = parseInt(remaining);
    }
    if (reset) {
      this.rateLimitReset = reset;
    }
  }

  /**
   * Make HTTP request to API
   */
  async makeRequest(method, path, data = null) {
    const url = `${this.baseUrl}${path}`;

    const options = {
      method,
      headers: {
        "X-API-Key": this.apiKey,
        "Content-Type": "application/json",
      },
    };

    if (data) {
      options.body = JSON.stringify(data);
    }

    try {
      const response = await fetch(url, options);

      // Update rate limit info
      this.updateRateLimitInfo(response);

      // Handle errors
      if (response.status === 429) {
        throw new Error(
          `Rate limit exceeded. Reset at ${this.rateLimitReset}`
        );
      }

      if (!response.ok) {
        const errorBody = await response.json().catch(() => ({}));
        throw new Error(
          `HTTP ${response.status}: ${errorBody.message || response.statusText}`
        );
      }

      return await response.json();
    } catch (error) {
      console.error(`API Error: ${error.message}`);
      throw error;
    }
  }

  /**
   * Check API health
   */
  async healthCheck() {
    return this.makeRequest("GET", "/health");
  }

  /**
   * Discover process model from event log
   *
   * @param {Array} events - List of events
   * @param {string} algorithm - Discovery algorithm (inductive, alpha, etc.)
   * @param {Object} parameters - Algorithm-specific parameters
   */
  async discover(events, algorithm = "inductive", parameters = {}) {
    const payload = {
      log: {
        events,
        format: "json",
      },
      algorithm,
    };

    if (Object.keys(parameters).length > 0) {
      payload.parameters = parameters;
    }

    return this.makeRequest("POST", "/discover", payload);
  }

  /**
   * Check log conformance against model
   *
   * @param {Array} events - List of events
   * @param {Object} model - Petri Net model
   * @param {string} variant - Conformance variant
   */
  async conform(events, model, variant = "token_replay") {
    const payload = {
      log: {
        events,
        format: "json",
      },
      model,
      variant,
    };

    return this.makeRequest("POST", "/conform", payload);
  }

  /**
   * Analyze model structure
   */
  async analyze(model) {
    return this.makeRequest("POST", "/analyze", { model });
  }

  /**
   * Extract log statistics
   */
  async stats(events) {
    const payload = {
      log: {
        events,
        format: "json",
      },
    };

    return this.makeRequest("POST", "/stats", payload);
  }
}

/**
 * Create sample event log for testing
 */
function createSampleEvents() {
  const events = [];
  const now = new Date();

  for (let caseNum = 1; caseNum <= 5; caseNum++) {
    const caseId = `loan_${String(caseNum).padStart(3, "0")}`;
    let caseTime = new Date(now);
    caseTime.setHours(caseTime.getHours() + caseNum);

    // Event 1: Apply
    events.push({
      case_id: caseId,
      activity: "apply",
      timestamp: caseTime.toISOString(),
      resource: "customer",
      attributes: { amount: 50000 * caseNum },
    });

    // Event 2: Register
    let time2 = new Date(caseTime);
    time2.setMinutes(time2.getMinutes() + 5);
    events.push({
      case_id: caseId,
      activity: "register",
      timestamp: time2.toISOString(),
      resource: "clerk",
    });

    // Event 3: Verify documents
    let time3 = new Date(caseTime);
    time3.setHours(time3.getHours() + 1);
    events.push({
      case_id: caseId,
      activity: "verify_documents",
      timestamp: time3.toISOString(),
      resource: "officer",
    });

    // Event 4: Credit check
    let time4 = new Date(caseTime);
    time4.setHours(time4.getHours() + 2);
    events.push({
      case_id: caseId,
      activity: "credit_check",
      timestamp: time4.toISOString(),
      resource: "system",
    });

    // Event 5: Approve
    let time5 = new Date(caseTime);
    time5.setHours(time5.getHours() + 3);
    events.push({
      case_id: caseId,
      activity: "approve",
      timestamp: time5.toISOString(),
      resource: "manager",
      attributes: { approved: true },
    });

    // Event 6: Disburse
    let time6 = new Date(caseTime);
    time6.setHours(time6.getHours() + 4);
    events.push({
      case_id: caseId,
      activity: "disburse",
      timestamp: time6.toISOString(),
      resource: "accountant",
      attributes: { amount_disbursed: 50000 * caseNum },
    });
  }

  return events;
}

/**
 * Format percentage
 */
function formatPercent(value) {
  return `${(value * 100).toFixed(1)}%`;
}

/**
 * Run examples
 */
async function main() {
  console.log("╔════════════════════════════════════════════════════════╗");
  console.log("║   PM4Py REST API Client Examples (JavaScript/Node.js)  ║");
  console.log("╚════════════════════════════════════════════════════════╝\n");

  // Get API key from environment
  const apiKey = process.env.PM4PY_API_KEY || "demo-key-for-testing";
  if (apiKey === "demo-key-for-testing") {
    console.log("⚠️  Using demo API key. Set PM4PY_API_KEY for real API.\n");
  }

  // Initialize client
  const client = new PM4PyClient(apiKey);

  try {
    // Example 1: Health check
    console.log("1. HEALTH CHECK");
    console.log(
      "─────────────────────────────────────────────────────────\n"
    );

    const health = await client.healthCheck();
    console.log(`✓ API Status: ${health.status}`);
    console.log(`  Version: ${health.version}`);
    console.log(`  Timestamp: ${health.timestamp}\n`);

    // Example 2: Process Discovery
    console.log("\n2. PROCESS DISCOVERY");
    console.log(
      "─────────────────────────────────────────────────────────\n"
    );

    const events = createSampleEvents();
    console.log(`Created event log with ${events.length} events\n`);

    const discoveryResult = await client.discover(events, "inductive", {
      frequency_threshold: 0.1,
    });

    const model = discoveryResult.model || {};
    console.log(`✓ Model discovered:`);
    console.log(`  Algorithm: ${discoveryResult.algorithm}`);
    console.log(`  Places: ${discoveryResult.num_places}`);
    console.log(`  Transitions: ${discoveryResult.num_transitions}`);
    console.log(`  Processing time: ${discoveryResult.processing_time_ms}ms`);
    console.log(
      `  Rate limit remaining: ${client.rateLimitRemaining} requests/hour\n`
    );

    // Example 3: Conformance Checking
    console.log("\n3. CONFORMANCE CHECKING");
    console.log(
      "─────────────────────────────────────────────────────────\n"
    );

    if (Object.keys(model).length > 0) {
      const conformanceResult = await client.conform(
        events,
        model,
        "token_replay"
      );

      const result = conformanceResult.result || {};
      console.log(`✓ Conformance checked:`);
      console.log(`  Fitness: ${formatPercent(result.fitness || 0)}`);
      console.log(`  Precision: ${formatPercent(result.precision || 0)}`);
      console.log(`  Generalization: ${formatPercent(result.generalization || 0)}`);
      console.log(`  Simplicity: ${formatPercent(result.simplicity || 0)}`);
      console.log(`  Deviant traces: ${(result.deviant_traces || []).length}`);
      console.log(`  Processing time: ${conformanceResult.processing_time_ms}ms\n`);
    }

    // Example 4: Log Statistics
    console.log("\n4. LOG STATISTICS");
    console.log(
      "─────────────────────────────────────────────────────────\n"
    );

    const statsResult = await client.stats(events);

    const stats = statsResult.stats || {};
    console.log(`✓ Statistics extracted:`);
    console.log(`  Traces: ${stats.num_traces}`);
    console.log(`  Events: ${stats.num_events}`);
    console.log(`  Activities: ${stats.num_activities}`);
    console.log(`  Mean trace length: ${(stats.trace_length_mean || 0).toFixed(2)}`);
    console.log(`  Mean case duration: ${stats.case_duration_mean}`);
    console.log(`  Processing time: ${statsResult.processing_time_ms}ms\n`);

    // Show top activities
    const activities = stats.activities || [];
    if (activities.length > 0) {
      console.log("  Top activities:");
      activities.slice(0, 3).forEach((activity) => {
        console.log(`    - ${activity.name}: ${activity.frequency} times`);
      });
    }

    // Example 5: Model Analysis
    if (Object.keys(model).length > 0) {
      console.log("\n5. MODEL ANALYSIS");
      console.log(
        "─────────────────────────────────────────────────────────\n"
      );

      const analysisResult = await client.analyze(model);

      console.log(`✓ Model analyzed:`);
      console.log(`  Is sound: ${analysisResult.is_sound}`);

      const profile = analysisResult.behavioral_profile || {};
      console.log(
        `  Strongly connected components: ${profile.strongly_connected_components}`
      );
      console.log(`  Longest path: ${profile.longest_path}`);

      const deadlocks = analysisResult.deadlock_potential || [];
      if (deadlocks.length > 0) {
        console.log(`  ⚠️  Potential deadlock configurations: ${deadlocks.length}`);
      } else {
        console.log(`  ✓ No deadlock potential`);
      }

      console.log(
        `  Processing time: ${analysisResult.processing_time_ms}ms\n`
      );
    }

    // Summary
    console.log(
      "\n╔════════════════════════════════════════════════════════╗"
    );
    console.log(
      "║                Examples Complete                       ║"
    );
    console.log(
      "╚════════════════════════════════════════════════════════╝\n"
    );

    console.log(
      `Rate limit status: ${client.rateLimitRemaining}/10000 remaining`
    );
    if (client.rateLimitReset) {
      console.log(`Rate limit resets: ${client.rateLimitReset}`);
    }
  } catch (error) {
    console.error(`\n✗ Error during examples: ${error.message}`);
    process.exit(1);
  }
}

// Run examples
main().catch(console.error);
