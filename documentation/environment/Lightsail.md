# Provision on AWS Lightsail

A virtual machine with both Docker and Platys installed can be easily provisioned using AWS Lightsail with a "one-click" install.

Navigate to the [AWS Console](http://console.aws.amazon.com) and login with your user. Click on the [Lightsail service](https://lightsail.aws.amazon.com/ls/webapp/home/instances).

![Alt Image Text](./images/lightsail-homepage.png "Lightsail Homepage")

## Provision instance

Click **Create instance** to navigate to the **Create an instance** dialog. 

![Alt Image Text](./images/lightsail-create-instance-1.png "Lightsail Homepage")

Optionally change the **Instance Location** to a AWS region of your liking.
Keep **Linux/Unix** for the **Select a platform** and click on **OS Only** and select **Ubuntu 22.04 LTS** for the **Select a blueprint**. 

![Alt Image Text](./images/lightsail-create-instance-2.png "Lightsail Homepage")

Scroll down to **Launch script** and add the following script 

```
export USERNAME=ubuntu
# ====> you might want to change the password to something more secure !!
export PASSWORD=abc123!
export PLATYS_VERSION=2.4.2
export NETWORK_NAME=eth0

# Prepare Environment Variables 
export PUBLIC_IP=$(curl ipinfo.io/ip)
export DOCKER_HOST_IP=$(ip addr show ${NETWORK_NAME} | grep "inet\b" | awk '{print $2}' | cut -d/ -f1)

# allow login by password
sudo sed -i "s/.*PasswordAuthentication.*/PasswordAuthentication yes/g" /etc/ssh/sshd_config
echo "${USERNAME}:${PASSWORD}"|chpasswd
sudo service sshd restart

# add alias "dataplatform" to /etc/hosts
echo "$DOCKER_HOST_IP     dataplatform" | sudo tee -a /etc/hosts

# Install Docker
sudo apt-get update
sudo apt-get install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release
sudo mkdir -p /etc/apt/keyrings    
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
sudo usermod -aG docker $USERNAME

# Install Docker Compose Switch
sudo curl -fL https://github.com/docker/compose-switch/releases/latest/download/docker-compose-linux-amd64 -o /usr/local/bin/compose-switch
chmod +x /usr/local/bin/compose-switch
sudo update-alternatives --install /usr/local/bin/docker-compose docker-compose /usr/local/bin/compose-switch 99

# Install Platys
sudo curl -L "https://github.com/TrivadisPF/platys/releases/download/${PLATYS_VERSION}/platys_${PLATYS_VERSION}_linux_x86_64.tar.gz" -o /tmp/platys.tar.gz
tar zvxf /tmp/platys.tar.gz 
sudo mv platys /usr/local/bin/
sudo chown root:root /usr/local/bin/platys
sudo rm /tmp/platys.tar.gz 

# Install ctop
sudo wget https://github.com/bcicen/ctop/releases/download/v0.7.7/ctop-0.7.7-linux-amd64 -O /usr/local/bin/ctop
sudo chmod +x /usr/local/bin/ctop

# Install wget, curl, jq
apt-get install -y wget curl jq tree

# Install kafkacat
apt-get install -y kafkacat

# needed for elasticsearch
sudo sysctl -w vm.max_map_count=262144   

# Make Environment Variables persistent
sudo echo "export PUBLIC_IP=$PUBLIC_IP" | sudo tee -a /etc/profile.d/platys-platform-env.sh
sudo echo "export DOCKER_HOST_IP=$DOCKER_HOST_IP" | sudo tee -a /etc/profile.d/platys-platform-env.sh
sudo echo "export COMPOSE_HTTP_TIMEOUT=300 | sudo tee -a /etc/profile.d/platys-platform-env.sh

# Source the environment
source /etc/profile.d/platys-platform-env.sh
```

into the **Launch Script** edit field
 
![Alt Image Text](./images/lightsail-create-instance-3.png "Lightsail Homepage")

Click on **Change SSH key pair** and leave the **Default** selected and then click on **Download** and save the file to a convenient location on your machine. Under **Choose your instance plan** select the size for your instance. Depending on what you later want to run inside your stack, you should choose an instance with at least **4GB** of RAM (of course the more docker containers you run, the more memory you will need).    

Under **Identify your instance** enter **Ubuntu-Analytics-Platform** into the edit field. 

![Alt Image Text](./images/lightsail-create-instance-4.png "Lightsail Homepage")

Click on **Create Instance** to start provisioning the instance. 

The new instance will show up in the Instances list on the Lightsail homepage. 

![Alt Image Text](./images/lightsail-image-started.png "Lightsail Homepage")

Click on the instance to navigate to the image details page. On the right you can find the Public IP address **18.196.124.212** of the newly created instance.

![Alt Image Text](./images/lightsail-image-details.png "Lightsail Homepage")

Next we have to configure the Firewall to allow traffic into the Lightsail instance. 

Click on the **Networking** tab/link to navigate to the network settings and under **Firewall** click on **+ Add another**.
We allow TCP traffic on ports 28000 - 28400 by selecting **Custom**, entering **28000 - 28400** into the **Port or Range** field and then click **Save**. 

![Alt Image Text](./images/lightsail-image-networking-add-firewall-rule.png "Lightsail Homepage")

Now let's see how the provisioning of the lightsail instance is doing. 
Navigate to the **Connect** tab and click **Connect using SSH** to open the console and enter the following command to watch the log file of the init script.

```
tail -f /var/log/cloud-init-output.log --lines 1000
```

The initialisation is finished when you see a line

```bash
Cloud-init v. 22.4.2-0ubuntu0~22.04.1 finished at Mon, 22 May 2023 10:07:43 +0000. Datasource DataSourceEc2Local.  Up 132.85 seconds
```

Check that `platys` has been install successfully by executing `platys -v`

```bash
ubuntu@ip-172-26-4-247:~$ platys -v
Platys - Trivadis Platform in a Box - v 2.4.2
https://github.com/trivadispf/platys
Copyright (c) 2018-2020, Trivadis AG
Usage:
  platys [flags]
  platys [command]

Available Commands:
  clean         Cleans the contents in the $PATH/container-volume folder
  gen           Generates all the needed artifacts for the docker-based modern (data) platform  
  help          Help about any command  
  init          Initializes the current directory to be the root for the Modern (Data) Platform by creating an initial config file, if one does not already exists  
  list_services List the services  
  stacks        Lists the predefined stacks available for the init command  
  version       Print the version number of platys

Flags:
  -h, --help      help for platys  
  -v, --verbose   verbose outputUse "platys [command] --help" for more information about a command.
  ubuntu@ip-172-26-4-247:~$
```

Optionally you can also SSH into the Lightsail instance using the **SSH key pair** you have downloaded above. For that open a terminal window (on Mac / Linux) or Putty (on Windows) and connect as ubuntu to the Public IP address of the instance.   

```
ssh -i LightsailDefaultKey-eu-central-1.pem ubuntu@18.196.124.212 
```

Your instance is now ready to use. Complete the post installation steps documented the [here](README.md).

## Stop an Instance

To stop the instance, navigate to the instance overview and click on the drop-down menu and select **Stop**. 

![Alt Image Text](./images/lightsail-stop-instance.png "Lightsail Homepage")

Click on **Stop** to confirm stopping the instance. 

![Alt Image Text](./images/lightsail-stop-instance-confirm.png "Lightsail Homepage")

A stopped instance will still incur charges, you have to delete the instance completely to stop charges. 

## Delete an Instance

t.b.d.

## Create a snapshot of an Instance

When an instance is stopped, you can create a snapshot, which you can keep, even if later drop the instance to reduce costs.

![Alt Image Text](./images/lightsail-image-create-snapshot.png "Lightsail Homepage")

You can always recreate an instance based on a snapshot. 

# De-provision the environment

To stop the environment, execute the following command:

```
docker-compose stop
```

after that it can be re-started using `docker-compose start`.

To stop and remove all running container, execute the following command:

```
docker-compose down
```

