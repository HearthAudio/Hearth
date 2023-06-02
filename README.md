![alt text](assets/logo.png)
<h1 align="center">
    Hearth
</h1>
<div align="center">
<img src="assets/beta.png" align="center" height="30" />
</div>
<img src="assets/spacer_vertical.png" align="center" height="30" />
<p align="center">
<img src="https://camo.githubusercontent.com/a91b4f5528c26920b6b8fa617267a7193a5bddd70a4d013dcf616c91af39fc22/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f636f7665726167652d38302532352d79656c6c6f77677265656e"/>
<img src="https://camo.githubusercontent.com/8733716e2fd7444a0f383a9e5f43779a016bae35ddde4e1cc32a4f90bd9bb775/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f76657273696f6e2d312e322e332d626c7565"/>
<img src="https://camo.githubusercontent.com/e15a935f2751eef0e60660dbf1186b2a27a3cc996b423872ec5249a70a97bfe7/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f646570656e64656e636965732d6f75742532306f66253230646174652d6f72616e6765"/>
<img src="https://camo.githubusercontent.com/107e932aa93175670d273c86cae8be4c96f156da7e13b9b8223b63fa23563fb0/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f636f646163792d422d677265656e"
</p>
<p align="center">
Hearth is a LavaLink Alternative written in Rust. That uses 30X less memory, and is almost in a state of feature parity. LavaLink is a audio streaming server for discord bots written in Java, that handles all of the complex audio processing required for music discord bots.
</p>
<h3 align="center">Features</h3>
<hr/>
<p align="center" >
<ul>
  <li>💨 Hearth is extremely performant and uses 30X less memory than LavaLink (See Benchmark section for more details)</li><br/>
  <li>📈 Hearth is designed from the ground up for massive scale and is horizontaly scalable.</li><br/>
  <li >🔧 Hearth is extremely easy to integrate into your project with a native Rust client library. And bindings for TS, Java, and Python coming soon!</li><br/>
</ul>
<div style="display: flex;align-content: center;justify-content: center;">
    <div style="display: flex;flex-direction: column;">
        <h3 align="center">Client Libraries</h3>
        <hr/>
        <div align="center">
            <h4>Coming Soon:</h4>
            <img height="70" src="assets/java.svg"/>
            <img  height="70"  src="assets/spacer.png"/>
            <img height="70" src="assets/python.png"/>
            <img  height="70"  src="assets/spacer.png"/>
            <img  height="70"  src="assets/javascript.svg"/>
            <img  height="70"  src="assets/spacer.png"/>
            <img  height="70"  src="assets/typescript.svg"/>
            <h4>Available Now:</h4>
            <img  height="70"  src="assets/spacer.png"/>
            <img  height="70"  src="assets/rust.svg"/>
        </div>
        <br/>
        <p align="center" >
            With our wide selection of pre-built client libraries you can get started on your bot super quickly and easily using whatever tool you want. 
        </p>
    </div>
</div>

<h3 align="center">Getting Started</h3>
<hr/>
<p align="center" >
Ready to get started with Hearth? See the getting started guide <a href="https://github.com/Hearth-Industries/Hearth/blob/master/GETTING_STARTED.md">here</a> to start a new project with Hearth or integrate Hearth into your pre-existing project.
</p>
<h3 align="center">Benchmarks</h3>
<hr/>
<p align="center" >
Hearth uses 30X less Memory than LavaLink, unfortunately Hearth has slightly worse CPU usage than LavaLink, due to inefficiencies
in the client library Hearth uses to interact with Discord. But this should be fixed in a future update! These numbers where derived by running Hearth in a DigitalOcean Droplet with 4 Intel CPU cores and 4GB of Memory. And observing CPU usage with `top` and memory usage with `ps`
</p>
<h3 align="center">Compatability</h3>
<hr/>
<p align="center" >
Note: Hearth does not support usage on Apple Silicon/ Hearth will fail silently if used on Apple Silicon. We recommend running Hearth in production on Ubuntu Linux on either an x86 platform or a non M series ARM chip.
<h3 align="center">Contributions</h3>
<hr/>
<p align="center" >
If you want to create an Issue, PR, Or contribute in any other way I'm happy to review PRs, or Issues.
</p>
<h3 align="center">Roadmap</h3>
<p align="center" >
We are planning tons of new features to make Hearth even better!
<hr/>
<p align="center" >
  CPU Performance Improvements<br/><br/>
  Effects<br/><br/>
  Dynamic Worker Joins to support Autoscaling nodes on platforms like AWS<br/><br/>
  Soundcloud Support<br/><br/>
  Dashboard Interface for Scheduler <br/><br/>
<h3 align="center">Licensing</h3>
<hr/>
<p align="center" >
Hearth is free to use under the MIT licnese.
</p>
